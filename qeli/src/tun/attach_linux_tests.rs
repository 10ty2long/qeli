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

// Mutation helpers below are called only inside isolated(), after unshare succeeds.
fn ip_link(args: &[&str]) -> io::Result<()> {
    let output = crate::system_command::Command::new("ip")
        .args(["link"])
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(())
}

fn persistent(tun: &TunInterface) -> io::Result<()> {
    // SAFETY: tun owns an attached descriptor; this ioctl takes an integer argument.
    if unsafe { libc::ioctl(tun.as_raw_fd(), libc::TUNSETPERSIST as _, 1) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, /dev/net/tun and ip; isolated netns"]
fn closing_renamed_device_preserves_persistent_replacement() -> io::Result<()> {
    isolated(|| {
        for kind in [DeviceType::Tun, DeviceType::Tap] {
            let name = if kind == DeviceType::Tun {
                "qeli-tun"
            } else {
                "qeli-tap"
            };
            let original =
                TunInterface::open_device(name, 1400, open::flags(kind, true, OpenMode::Create))?;
            ip_link(&["set", "dev", name, "name", "qeli-moved"])?;
            let replacement =
                TunInterface::open_device(name, 1400, open::flags(kind, true, OpenMode::Create))?;
            persistent(&replacement)?;
            let index = open::interface_index(name)?.expect("replacement must exist");
            drop(original);
            assert_eq!(open::interface_index("qeli-moved")?, None);
            drop(replacement);
            // A name-based ip tuntap del would have cleared this device's persistence.
            assert_eq!(open::interface_index(name)?, Some(index));
        }
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, /dev/net/tun and ip; isolated netns"]
fn closing_detached_queues_preserves_persistent_replacement() -> io::Result<()> {
    isolated(|| {
        let name = "qeli-audit0";
        let original = TunInterface::create_multiqueue(name, 1400, DeviceType::Tun, 3)?;
        ip_link(&["del", "dev", name])?;
        assert_eq!(open::interface_index(name)?, None);
        let replacement = TunInterface::create_multiqueue(name, 1400, DeviceType::Tun, 2)?;
        persistent(&replacement[0])?;
        let index = open::interface_index(name)?.expect("replacement must exist");
        drop(original);
        drop(replacement);
        assert_eq!(open::interface_index(name)?, Some(index));
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN and /dev/net/tun; isolated netns"]
fn duplicate_workers_close_before_original_descriptor_releases_name() -> io::Result<()> {
    isolated(|| {
        let name = "qeli-audit0";
        let mut owner = TunInterface::create_multiqueue(name, 1400, DeviceType::Tun, 3)?;
        let readers: Vec<_> = owner
            .iter()
            .map(|queue| queue.fd.try_clone())
            .collect::<io::Result<_>>()?;
        let writers: Vec<_> = owner
            .iter()
            .map(|queue| queue.fd.try_clone())
            .collect::<io::Result<_>>()?;
        owner.truncate(1);
        let index = open::interface_index(name)?.expect("created device must exist");
        drop(readers);
        drop(writers);
        assert_eq!(open::interface_index(name)?, Some(index));
        // A competing generation must still be refused throughout host cleanup.
        assert!(TunInterface::create(name, 1400).is_err());
        drop(owner);
        assert_eq!(open::interface_index(name)?, None);
        drop(TunInterface::create(name, 1400)?);
        Ok(())
    })
}
