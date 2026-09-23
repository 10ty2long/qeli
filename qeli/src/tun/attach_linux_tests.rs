//! Opt-in real ioctl checks. Every test enters a private network namespace on
//! a disposable OS thread before opening or creating any TUN/TAP.
use super::*;

fn isolated(test: fn() -> io::Result<()>) -> io::Result<()> {
    std::thread::spawn(move || {
        // SAFETY: only this fresh thread changes its network namespace. No descriptor
        // or network mutation is attempted unless isolation succeeded.
        if unsafe { libc::unshare(libc::CLONE_NEWNET) } != 0 {
            return Err(io::Error::last_os_error());
        }
        test()
    })
    .join()
    .expect("isolated TUN test thread panicked")
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN and /dev/net/tun; isolated netns"]
fn missing_attach_never_registers_tun_or_tap() -> io::Result<()> {
    isolated(|| {
        // Prove device access and creation privileges before testing a refusal.
        // Otherwise a missing /dev/net/tun could make this test pass vacuously.
        drop(TunInterface::create("qeli-probe", 1400)?);
        for kind in [DeviceType::Tun, DeviceType::Tap] {
            for mq in [false, true] {
                let name = "qeli-audit0";
                assert_eq!(open::interface_index(name)?, None);
                assert!(TunInterface::open_device(
                    name,
                    1400,
                    open::flags(kind, mq, OpenMode::Attach)
                )
                .is_err());
                assert_eq!(open::interface_index(name)?, None);
            }
        }
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN and /dev/net/tun; isolated netns"]
fn attachment_guard_keeps_existing_device_and_releases_only_its_fd() -> io::Result<()> {
    isolated(|| {
        for kind in [DeviceType::Tun, DeviceType::Tap] {
            let name = "qeli-audit0";
            let owner =
                TunInterface::open_device(name, 1400, open::flags(kind, true, OpenMode::Create))?;
            let index = open::interface_index(name)?.expect("created device must exist");
            let borrower =
                TunInterface::open_device(name, 1400, open::flags(kind, true, OpenMode::Attach))?;
            assert_eq!(open::interface_index(name)?, Some(index));
            drop(borrower);
            assert_eq!(open::interface_index(name)?, Some(index));
            drop(owner);
            assert_eq!(open::interface_index(name)?, None);
        }
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN and /dev/net/tun; isolated netns"]
fn multiqueue_creation_uses_one_device_and_last_fd_releases_it() -> io::Result<()> {
    isolated(|| {
        for kind in [DeviceType::Tun, DeviceType::Tap] {
            let queues = TunInterface::create_multiqueue("qeli-audit%d", 1400, kind, 3)?;
            let name = queues[0].name.clone();
            let index = open::interface_index(&name)?.expect("created device must exist");
            assert!(queues.iter().all(|queue| queue.name == name));
            let mut queues = queues.into_iter();
            drop(queues.next());
            assert_eq!(open::interface_index(&name)?, Some(index));
            drop(queues);
            assert_eq!(open::interface_index(&name)?, None);
        }
        Ok(())
    })
}
