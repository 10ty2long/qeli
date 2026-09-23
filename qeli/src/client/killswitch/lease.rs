//! Cooperative lifetime exclusion for protected Linux clients in one network namespace.
//!
//! Linux scopes abstract AF_UNIX names to the network namespace. Atomic bind gives
//! immediate exclusion without lock files, PID reuse, stale inode removal or waiting
//! for another process. Closing the last descriptor releases the name, including on
//! process death. Rust sockets are close-on-exec, so credential/hooks/iptables children
//! cannot retain the lease after the client exits. No messages are sent or received.
//!
//! This is not firewall cleanup: a crash still leaves DROP rules for recovery. Old
//! Qeli binaries/admin tools do not cooperate; filter admission remains necessary.
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

// Stable across versions, configs, TUN names, users, mount namespaces and state dirs.
// Changing this name would let different versions concurrently own the same firewall.
const NAME: &[u8] = b"qeli.client.kill-switch";

#[derive(Debug)]
pub(crate) struct Lease {
    _socket: UnixDatagram,
}

pub(crate) fn acquire() -> io::Result<Lease> {
    bind(NAME)
}

fn bind(name: &[u8]) -> io::Result<Lease> {
    let address = SocketAddr::from_abstract_name(name)?;
    Ok(Lease {
        _socket: UnixDatagram::bind_addr(&address)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn name() -> Vec<u8> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        format!(
            "qeli-test-lease-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
        .into_bytes()
    }

    #[test]
    fn live_owner_blocks_reentry_and_drop_allows_recovery() {
        let name = name();
        let owner = bind(&name).unwrap();
        assert_eq!(bind(&name).unwrap_err().kind(), io::ErrorKind::AddrInUse);
        drop(owner);
        let _replacement = bind(&name).unwrap();
    }

    #[test]
    fn independent_names_do_not_block_each_other() {
        let _first = bind(&name()).unwrap();
        let _second = bind(&name()).unwrap();
    }

    #[test]
    fn concurrent_claims_admit_exactly_one_owner() {
        let name = name();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let children: Vec<_> = (0..2)
            .map(|_| {
                let name = name.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    bind(&name)
                })
            })
            .collect();
        // Keep the winner's descriptor live until both attempts have completed.
        let owners: Vec<_> = children
            .into_iter()
            .map(|child| child.join().unwrap())
            .collect();
        assert_eq!(owners.iter().filter(|owner| owner.is_ok()).count(), 1);
        assert_eq!(
            owners
                .iter()
                .filter(|owner| owner
                    .as_ref()
                    .is_err_and(|error| error.kind() == io::ErrorKind::AddrInUse))
                .count(),
            1
        );
    }

    #[test]
    fn unwinding_releases_lease_without_network_cleanup() {
        let name = name();
        let failure = std::panic::catch_unwind(|| {
            let _owner = bind(&name).unwrap();
            panic!("simulated client failure");
        });
        assert!(failure.is_err());
        let _replacement = bind(&name).unwrap();
    }

    #[test]
    fn descriptor_is_close_on_exec() {
        use std::os::fd::AsRawFd;
        let owner = bind(&name()).unwrap();
        // SAFETY: the borrowed descriptor is live; F_GETFD has no extra argument.
        let flags = unsafe { libc::fcntl(owner._socket.as_raw_fd(), libc::F_GETFD) };
        assert!(flags >= 0);
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
    }

    // A child fixture, never an ordinary test (it terminates its own test process).
    #[test]
    #[ignore]
    fn lease_child() {
        let name = std::env::var("QELI_TEST_LEASE_NAME").unwrap();
        let _owner = bind(name.as_bytes());
        std::process::exit(if _owner.is_ok() { 0 } else { 7 });
    }

    fn child(name: &[u8]) -> std::process::ExitStatus {
        let test = format!(
            "{}::lease_child",
            module_path!().split_once("::").unwrap().1
        );
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--ignored", "--nocapture"])
            .env("QELI_TEST_LEASE_NAME", std::str::from_utf8(name).unwrap())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                return status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("lease child timed out");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn another_process_is_excluded_and_process_exit_releases_lease() {
        let name = name();
        let owner = bind(&name).unwrap();
        assert_eq!(child(&name).code(), Some(7));
        drop(owner);
        assert_eq!(child(&name).code(), Some(0));
        let _replacement = bind(&name).unwrap();
    }
}
