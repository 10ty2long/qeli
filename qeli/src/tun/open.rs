//! Shared Linux TUN/TAP opening policy, independent of ioctl transport.
use std::io;
pub(super) const IFF_TUN: i16 = 0x0001;
pub(super) const IFF_TAP: i16 = 0x0002;
pub(super) const IFF_NO_PI: i16 = 0x1000;
pub(super) const IFF_MULTI_QUEUE: i16 = 0x0100;
const IFF_TUN_EXCL: i16 = 0x8000u16 as i16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    Tun,
    Tap,
}
#[derive(Clone, Copy)]
pub(super) enum OpenMode {
    Create,
    Attach,
}
pub(super) fn flags(kind: DeviceType, multiqueue: bool, mode: OpenMode) -> i16 {
    let mut flags = match kind {
        DeviceType::Tun => IFF_TUN,
        DeviceType::Tap => IFF_TAP,
    } | IFF_NO_PI;
    if multiqueue {
        flags |= IFF_MULTI_QUEUE;
    }
    if matches!(mode, OpenMode::Create) {
        flags |= IFF_TUN_EXCL;
    }
    flags
}
/// First queue creates exclusively; subsequent queues use the name returned by it.
/// Dropping the vector on any later error closes all queues already acquired.
pub(super) fn create_queues<T>(
    name: &str,
    kind: DeviceType,
    count: usize,
    mut open: impl FnMut(&str, i16) -> io::Result<T>,
    name_of: impl Fn(&T) -> &str,
) -> io::Result<Vec<T>> {
    let count = count.max(1);
    let mut queues = Vec::with_capacity(count);
    queues.push(open(name, flags(kind, true, OpenMode::Create))?);
    for _ in 1..count {
        queues.push(open(
            name_of(&queues[0]),
            flags(kind, true, OpenMode::Attach),
        )?);
    }
    Ok(queues)
}

/// Query the calling thread's network namespace, without relying on a sysfs mount.
#[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
pub(crate) fn interface_index(name: &str) -> io::Result<Option<u32>> {
    let name = std::ffi::CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "interface name contains NUL"))?;
    // SAFETY: name is a valid, NUL-terminated string for the duration of the call.
    let index = unsafe { libc::if_nametoindex(name.as_ptr()) };
    if index != 0 {
        return Ok(Some(index));
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENODEV) {
        Ok(None)
    } else {
        Err(error)
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod tests;
