//! A filename or a commented stub address cannot prove the system DNS path.
use std::{fs::OpenOptions, io::Read, os::unix::fs::OpenOptionsExt, path::Path};
const LIMIT: u64 = 64 * 1024;

pub(super) fn uses_stub(path: &Path) -> bool {
    read_stub(path).unwrap_or(false)
}
fn read_stub(path: &Path) -> std::io::Result<bool> {
    // Follow the normal resolv.conf symlink, but validate/read one opened regular file.
    // NONBLOCK avoids hanging on a FIFO substituted for the configuration file.
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > LIMIT {
        return Ok(false);
    }
    let mut contents = String::new();
    file.by_ref()
        .take(LIMIT + 1)
        .read_to_string(&mut contents)?;
    let after = file.metadata()?;
    if contents.len() as u64 > LIMIT
        || contents.len() as u64 != after.len()
        || crate::config_source::Stamp::of(&before) != crate::config_source::Stamp::of(&after)
    {
        return Ok(false);
    }
    Ok(stub_only(&contents))
}
fn stub_only(contents: &str) -> bool {
    if contents.contains('\0') {
        return false;
    }
    let mut found = false;
    for line in contents.lines() {
        let mut fields = line
            .split(['#', ';'])
            .next()
            .unwrap_or_default()
            .split_ascii_whitespace();
        if fields.next() != Some("nameserver") {
            continue;
        }
        if !matches!(fields.next(), Some("127.0.0.53" | "127.0.0.54")) || fields.next().is_some() {
            return false;
        }
        found = true;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolver_config_requires_actual_stub_nameservers_only() {
        for value in [
            "nameserver 127.0.0.53\n",
            "  nameserver\t127.0.0.54 # proxy\nsearch example.test\n",
            "nameserver 127.0.0.53\nnameserver 127.0.0.54\n",
        ] {
            assert!(stub_only(value), "{value:?}");
        }
        for value in [
            "",
            "# nameserver 127.0.0.53\nnameserver 192.0.2.53",
            "search 127.0.0.53",
            "nameserver 127.0.0.530",
            "nameserver 127.0.0.53\nnameserver 192.0.2.53",
            "nameserver 127.0.0.53 extra",
            "nameserver\n",
            "nameserver 127.0.0.53\0\nnameserver 192.0.2.53",
        ] {
            assert!(!stub_only(value), "{value:?}");
        }
    }
    #[test]
    fn resolver_config_symlink_name_never_bypasses_content_check() {
        use std::os::unix::fs::symlink;
        let tmp = super::super::tests::Tmp::new("resolver-config");
        let target = tmp.path("stub-resolv.conf");
        let link = tmp.path("resolv.conf");
        symlink(&target, &link).unwrap();
        assert!(!uses_stub(&link), "dangling link");
        std::fs::write(&target, "nameserver 192.0.2.53\n").unwrap();
        assert!(!uses_stub(&link), "stub-like name with real upstream");
        std::fs::write(&target, "nameserver 127.0.0.53\n").unwrap();
        assert!(uses_stub(&link));
        std::fs::write(&target, vec![b'#'; LIMIT as usize + 1]).unwrap();
        assert!(!uses_stub(&link));
        std::fs::write(&target, [0xff]).unwrap();
        assert!(!uses_stub(&link));
    }
    #[test]
    fn resolver_config_fifo_is_refused_without_waiting_for_writer() {
        use std::{
            ffi::CString,
            os::unix::ffi::OsStrExt,
            time::{Duration, Instant},
        };
        let tmp = super::super::tests::Tmp::new("resolver-fifo");
        let path = tmp.path("fifo");
        let c = CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: path is NUL-terminated and lives through this syscall.
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        let started = Instant::now();
        assert!(!uses_stub(&path));
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
