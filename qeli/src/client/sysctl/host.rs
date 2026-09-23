//! Injectable process/sysctl I/O. Host regressions never read or modify real procfs.
use std::io;
pub(super) fn read(path: &str) -> io::Result<String> {
    #[cfg(test)]
    {
        test_support::call(test_support::Operation::Read(path))
    }
    #[cfg(not(test))]
    {
        std::fs::read_to_string(path)
    }
}
pub(super) fn write(path: &str, value: &str) -> io::Result<()> {
    #[cfg(test)]
    {
        test_support::call(test_support::Operation::Write(path, value)).map(|_| ())
    }
    #[cfg(not(test))]
    {
        std::fs::write(path, value)
    }
}
pub(super) fn process_exists(pid: u32) -> io::Result<bool> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid process ID",
        ));
    }
    #[cfg(test)]
    {
        test_support::call(test_support::Operation::ProcessExists(pid)).map(|value| value == "1")
    }
    #[cfg(all(not(test), target_os = "linux"))]
    {
        // Signal 0 performs an existence/permission check; it delivers no signal.
        // SAFETY: pid is a positive representable process ID, not a process group.
        if unsafe { libc::kill(pid as libc::pid_t, 0) } == 0 {
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ESRCH) => Ok(false),
            Some(libc::EPERM) => Ok(true),
            _ => Err(error),
        }
    }
}
pub(super) fn interface_exists(name: &str) -> io::Result<bool> {
    #[cfg(test)]
    {
        test_support::call(test_support::Operation::InterfaceExists(name)).map(|value| value == "1")
    }
    #[cfg(not(test))]
    {
        // A complete successful inventory proves absence. A missing/denied sysfs
        // root or an iteration error must not discard saved recovery values.
        for entry in std::fs::read_dir("/sys/class/net")? {
            if entry?.file_name() == name {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use std::{cell::RefCell, io};
    pub(crate) enum Operation<'a> {
        Read(&'a str),
        Write(&'a str, &'a str),
        ProcessExists(u32),
        InterfaceExists(&'a str),
    }
    type Handler = Box<dyn FnMut(Operation<'_>) -> io::Result<String>>;
    thread_local! {
        static BACKEND: RefCell<Option<Handler>> = RefCell::new(None);
    }
    pub(super) fn call(operation: Operation<'_>) -> io::Result<String> {
        BACKEND.with(|slot| {
            slot.borrow_mut()
                .as_mut()
                .expect("sysctl host tests require isolated I/O")(operation)
        })
    }
    pub(crate) fn with_io<T>(
        handler: impl FnMut(Operation<'_>) -> io::Result<String> + 'static,
        run: impl FnOnce() -> T,
    ) -> T {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                BACKEND.with(|slot| *slot.borrow_mut() = None);
            }
        }
        BACKEND.with(|slot| {
            assert!(slot.borrow().is_none(), "nested sysctl I/O override");
            *slot.borrow_mut() = Some(Box::new(handler));
        });
        let _reset = Reset;
        run()
    }
}
