//! Explicit privileged tests. Every mutation follows CLONE_NEWNET in a new thread.
use super::*;
use std::os::fd::AsRawFd;

fn new_namespace() -> anyhow::Result<()> {
    // SAFETY: caller is a disposable test thread, never an application worker.
    if unsafe { libc::unshare(libc::CLONE_NEWNET) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
fn command(program: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(program).args(args).output()?;
    anyhow::ensure!(
        output.status.success(),
        "{program} {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}
fn tools() -> anyhow::Result<[String; 2]> {
    Ok([
        ipt_path("iptables").ok_or_else(|| anyhow::anyhow!("iptables required"))?,
        ipt_path("ip6tables").ok_or_else(|| anyhow::anyhow!("ip6tables required"))?,
    ])
}

#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN and iptables/ip6tables; isolated netns"]
fn native_kill_switch_namespace_change_preserves_foreign_rules() -> anyhow::Result<()> {
    std::thread::spawn(|| -> anyhow::Result<()> {
        new_namespace()?;
        let original = std::fs::File::open("/proc/thread-self/ns/net")?;
        let paths = tools()?;
        let tun = "ks_native0";
        let chain = chain_for(tun);
        engage("203.0.113.7", 443, tun, false, false, true)?;
        for path in &paths {
            anyhow::ensure!(present_checked(path, &["-C", "OUTPUT", "-j", &chain])?);
        }
        new_namespace()?;
        for path in &paths {
            command(path, &["-N", &chain])?;
            command(path, &["-A", &chain, "-m", "comment", "--comment", "foreign-sentinel", "-j", "DROP"])?;
            command(path, &["-I", "OUTPUT", "-j", &chain])?;
        }
        // Exercise the public entry points, not just a namespace comparison helper.
        let refresh = refresh_server_ips("203.0.113.8", 443, tun);
        let cleanup = disengage(tun);
        let restart = engage("203.0.113.8", 443, tun, true, true, false);
        let preserved = paths.iter().all(|path| {
            present_checked(path, &["-C", &chain, "-m", "comment", "--comment", "foreign-sentinel", "-j", "DROP"]).unwrap_or(false)
        });
        // SAFETY: return this same disposable thread to its still-open original netns.
        if unsafe { libc::setns(original.as_raw_fd(), libc::CLONE_NEWNET) } != 0 { return Err(std::io::Error::last_os_error().into()); }
        disengage(tun)?;
        anyhow::ensure!(refresh.is_err() && cleanup.is_err() && restart.is_err() && preserved,
            "foreign namespace touched: refresh={refresh:?}, cleanup={cleanup:?}, restart={restart:?}, preserved={preserved}");
        for path in &paths { anyhow::ensure!(!chain_exists(&Context::fixture(), path, &chain)?); }
        Ok(())
    }).join().expect("native kill-switch namespace test panicked")
}

fn packets(
    path: &str,
    chain: &str,
    target: &str,
    destination: Option<&str>,
) -> anyhow::Result<u64> {
    let text = command(path, &["-n", "-v", "-x", "-L", chain])?;
    let mut count = 0;
    let mut matched = false;
    for line in text.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() >= 3
            && fields[2] == target
            && destination.is_none_or(|address| fields.contains(&address))
        {
            count += fields[0].parse::<u64>()?;
            matched = true;
        }
    }
    anyhow::ensure!(
        matched,
        "counter rule missing: {target} {destination:?}: {text}"
    );
    Ok(count)
}
fn probe(address: &str, ipv6: bool) -> anyhow::Result<()> {
    // No peer exists on the dummy WAN: an echo may fail, but filter counters must move.
    let output = Command::new("ping")
        .args([
            if ipv6 { "-6" } else { "-4" },
            "-n",
            "-c",
            "1",
            "-W",
            "1",
            address,
        ])
        .output()?;
    anyhow::ensure!(
        matches!(output.status.code(), Some(0 | 1)),
        "ping fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
#[ignore = "requires Linux CAP_SYS_ADMIN/CAP_NET_ADMIN, ip, ping and iptables/ip6tables; isolated netns"]
fn native_kill_switch_packets_stay_blocked_across_tun_reconnect() -> anyhow::Result<()> {
    std::thread::spawn(|| -> anyhow::Result<()> {
        new_namespace()?;
        command("ip", &["link", "set", "lo", "up"])?;
        command("ip", &["link", "add", "wan0", "type", "dummy"])?;
        command("ip", &["link", "set", "wan0", "up"])?;
        command("ip", &["addr", "add", "192.0.2.1/24", "dev", "wan0"])?;
        command(
            "ip",
            &[
                "-6",
                "addr",
                "add",
                "2001:db8::1/64",
                "dev",
                "wan0",
                "nodad",
            ],
        )?;
        command("ip", &["route", "add", "default", "dev", "wan0"])?;
        command("ip", &["-6", "route", "add", "default", "dev", "wan0"])?;
        let paths = tools()?;
        let tun = "ks_native1";
        let chain = chain_for(tun);
        for (ipv6, path, old, new, blocked) in [
            (
                false,
                &paths[0],
                "203.0.113.7",
                "203.0.113.8",
                "203.0.113.9",
            ),
            (
                true,
                &paths[1],
                "2001:db8:1::7",
                "2001:db8:1::8",
                "2001:db8:1::9",
            ),
        ] {
            engage(old, 443, tun, false, false, true)?;
            probe(old, ipv6)?;
            anyhow::ensure!(
                packets(path, &chain, "ACCEPT", Some(old))? > 0,
                "server path blocked"
            );
            probe(blocked, ipv6)?;
            let drops = packets(path, &chain, "DROP", None)?;
            anyhow::ensure!(drops > 0, "unapproved WAN traffic not dropped");
            command("ip", &["link", "add", tun, "type", "dummy"])?;
            command("ip", &["link", "set", tun, "up"])?;
            command("ip", &["link", "del", tun])?;
            refresh_server_ips(new, 443, tun)?;
            anyhow::ensure!(!present_checked(
                path,
                &["-C", &chain, "-d", old, "-j", "ACCEPT"]
            )?);
            probe(new, ipv6)?;
            anyhow::ensure!(packets(path, &chain, "ACCEPT", Some(new))? > 0);
            probe(blocked, ipv6)?;
            anyhow::ensure!(
                packets(path, &chain, "DROP", None)? > drops,
                "reconnect opened WAN egress"
            );
            disengage(tun)?;
            for path in &paths {
                anyhow::ensure!(!chain_exists(&Context::fixture(), path, &chain)?);
            }
        }
        Ok(())
    })
    .join()
    .expect("native kill-switch packet test panicked")
}
