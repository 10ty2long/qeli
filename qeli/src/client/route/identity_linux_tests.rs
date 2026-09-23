//! Native regressions: no network mutation until a disposable thread has entered
//! a fresh network namespace. Missing privileges/tools are errors, never skips.
use super::*;
use crate::tun::iface::TunInterface;
use std::os::fd::AsRawFd;

fn isolated(test: fn() -> anyhow::Result<()>) -> anyhow::Result<()> {
    let _serial = ROUTE_TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    journal::reset_tests();
    let result = std::thread::spawn(move || {
        // SAFETY: affects only this newly created test thread.
        if unsafe { libc::unshare(libc::CLONE_NEWNET) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        test()
    })
    .join()
    .expect("route identity test panicked");
    journal::reset_tests();
    result
}
fn ip(args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("ip").args(args).output()?;
    anyhow::ensure!(
        output.status.success(),
        "ip {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}
fn dummy(name: &str) -> anyhow::Result<()> {
    ip(&["link", "add", name, "type", "dummy"])?;
    ip(&["link", "set", name, "up"])?;
    Ok(())
}
fn add_owned(owner: &RouteOwner, prefix: &str, dev: &str) -> anyhow::Result<()> {
    ip(&["route", "add", prefix, "dev", dev])?;
    note_created_owned(
        owner,
        ["route", "del", prefix, "dev", dev]
            .map(str::to_string)
            .to_vec(),
    );
    Ok(())
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, ip and /dev/net/tun; isolated netns"]
fn native_cleanup_rename_preserves_replacement_and_cleans_physical_bypass() -> anyhow::Result<()> {
    isolated(|| {
        let owner = RouteOwner::new("qeli-audit0", 1)?;
        let tun = TunInterface::create(owner.interface(), 1400)?;
        owner.bind_tun(&tun)?;
        ip(&["link", "set", owner.interface(), "up"])?;
        add_owned(&owner, "10.41.0.0/16", owner.interface())?;
        dummy("qeli-physical")?;
        add_owned(&owner, "10.42.0.0/16", "qeli-physical")?;
        ip(&["link", "set", owner.interface(), "down"])?;
        ip(&["link", "set", owner.interface(), "name", "qeli-renamed"])?;
        dummy(owner.interface())?;
        ip(&["route", "add", "10.43.0.0/16", "dev", owner.interface()])?;
        assert!(cleanup_routes_for_tun(&owner, &tun).is_err());
        assert!(ip(&["route", "show", "exact", "10.42.0.0/16"])?
            .trim()
            .is_empty());
        assert!(ip(&["route", "show", "exact", "10.43.0.0/16"])?.contains("qeli-audit0"));
        assert!(ip(&["route", "show", "exact", "10.41.0.0/16"])?.contains("qeli-renamed"));
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, ip and /dev/net/tun; isolated netns"]
fn native_cleanup_deleted_original_does_not_delete_same_name_route() -> anyhow::Result<()> {
    isolated(|| {
        let owner = RouteOwner::new("qeli-audit0", 1)?;
        let tun = TunInterface::create(owner.interface(), 1400)?;
        owner.bind_tun(&tun)?;
        ip(&["link", "set", owner.interface(), "up"])?;
        add_owned(&owner, "10.41.0.0/16", owner.interface())?;
        ip(&["link", "del", owner.interface()])?;
        dummy(owner.interface())?;
        ip(&["route", "add", "10.41.0.0/16", "dev", owner.interface()])?;
        assert!(cleanup_routes_for_tun(&owner, &tun).is_err());
        assert!(ip(&["route", "show", "exact", "10.41.0.0/16"])?.contains("qeli-audit0"));
        Ok(())
    })
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, ip and /dev/net/tun; isolated netns"]
fn native_cleanup_namespace_change_preserves_foreign_physical_route() -> anyhow::Result<()> {
    isolated(|| {
        let original_namespace = std::fs::File::open("/proc/thread-self/ns/net")?;
        let owner = RouteOwner::new("qeli-audit0", 1)?;
        let tun = TunInterface::create(owner.interface(), 1400)?;
        owner.bind_tun(&tun)?;
        assert!(owner.bind_tun(&tun).is_err(), "an owner cannot be rebound");
        dummy("qeli-physical")?;
        add_owned(&owner, "10.42.0.0/16", "qeli-physical")?;
        // SAFETY: still the disposable isolated test thread.
        if unsafe { libc::unshare(libc::CLONE_NEWNET) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        dummy("qeli-physical")?;
        ip(&["route", "add", "10.42.0.0/16", "dev", "qeli-physical"])?;
        assert!(cleanup_routes_for_tun(&owner, &tun).is_err());
        assert!(ip(&["route", "show", "exact", "10.42.0.0/16"])?.contains("qeli-physical"));
        // SAFETY: return only to the private namespace held by our own descriptor.
        if unsafe { libc::setns(original_namespace.as_raw_fd(), libc::CLONE_NEWNET) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        cleanup_routes_for_tun(&owner, &tun)?;
        assert!(ip(&["route", "show", "exact", "10.42.0.0/16"])?
            .trim()
            .is_empty());
        Ok(())
    })
}
