//! Host-safe integration of the firewall helpers and the real bounded collector.
use super::*;
use crate::system_command::test_support::{arguments, with_commands, Action};
use std::io;

fn ipt_probe(mode: &'static str, expected: io::ErrorKind) {
    with_commands(
        move |command| {
            assert_eq!(command.get_program(), "qeli-test-iptables");
            assert_eq!(arguments(command), ["-C", "OUTPUT", "-j", "QELI_KS_qtest"]);
            Action::Probe { mode }
        },
        || {
            let result = ipt(
                "qeli-test-iptables",
                &["-C", "OUTPUT", "-j", "QELI_KS_qtest"],
            );
            assert!(matches!(result, Err(ref error) if error.kind() == expected));
        },
    );
}
#[test]
fn command_bounds_iptables_deadline_rejects_late_success() {
    ipt_probe("slow", io::ErrorKind::TimedOut);
}
#[test]
fn command_bounds_iptables_overflow_is_an_error() {
    ipt_probe("bytes", io::ErrorKind::InvalidData);
}
#[test]
fn command_bounds_firewall_queries_never_claim_absence_on_limit() {
    for mode in ["slow", "bytes"] {
        with_commands(
            move |command| {
                assert_eq!(command.get_program(), "qeli-test-iptables");
                Action::Probe { mode }
            },
            || {
                assert!(present_checked(
                    "qeli-test-iptables",
                    &["-C", "OUTPUT", "-j", "QELI_KS_qtest"]
                )
                .is_err());
                assert!(chain_exists("qeli-test-iptables", "QELI_KS_qtest").is_err());
            },
        );
    }
}
#[test]
fn command_bounds_teardown_does_not_flush_uninspectable_chain() {
    for mode in ["slow", "bytes"] {
        with_commands(
            move |command| {
                assert_eq!(command.get_program(), "qeli-test-iptables");
                assert!(
                    matches!(arguments(command)[0].as_str(), "-C" | "-S"),
                    "no blind firewall mutation"
                );
                Action::Probe { mode }
            },
            || assert!(teardown_family("qeli-test-iptables", "QELI_KS_qtest").is_err()),
        );
    }
}
#[test]
fn command_bounds_server_allow_inventory_rejects_partial_output() {
    for mode in ["slow", "bytes"] {
        with_commands(
            move |command| {
                assert_eq!(command.get_program(), "qeli-test-iptables");
                assert_eq!(arguments(command), ["-S", "QELI_KS_qtest"]);
                Action::Probe { mode }
            },
            || assert!(live_server_allows("qeli-test-iptables", "QELI_KS_qtest").is_err()),
        );
    }
}
#[test]
fn command_bounds_firewall_preserves_complete_success_and_nonzero_exit() {
    for (mode, success) in [("stdin", true), ("fail", false)] {
        with_commands(
            move |_| Action::Probe { mode },
            || {
                let result = ipt("qeli-test-iptables", &["--version"]).unwrap();
                assert_eq!(result.status.success(), success);
                if !success {
                    assert_eq!(result.status.code(), Some(17));
                    assert_eq!(result.stderr, b"expected command failure");
                }
            },
        );
    }
}

#[test]
fn command_bounds_unknown_ipv4_route_keeps_protection_required() {
    for mode in ["slow", "bytes"] {
        with_commands(
            move |command| {
                assert_eq!(command.get_program(), "ip");
                assert_eq!(arguments(command), ["-4", "route", "show", "default"]);
                Action::Probe { mode }
            },
            || {
                assert!(
                    host_may_have_ipv4_default_route(),
                    "unknown is not an IPv6-only host"
                )
            },
        );
    }
}

fn query_output(success: bool, stdout: &[u8]) -> Action {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    Action::Reply(Ok(std::process::Output {
        status: std::process::ExitStatus::from_raw(if success { 0 } else { 256 }),
        stdout: stdout.to_vec(),
        stderr: Vec::new(),
    }))
}
#[test]
fn command_bounds_only_successful_empty_ipv4_query_skips_protection() {
    for (success, stdout, required) in [
        (true, &b""[..], false),
        (true, &b"default via 192.0.2.1 dev eth0\n"[..], true),
        (false, &b""[..], true),
        (false, &b"default via 192.0.2.1 dev eth0\n"[..], true),
        (true, &b"\xff"[..], true),
    ] {
        with_commands(
            move |_| query_output(success, stdout),
            || assert_eq!(host_may_have_ipv4_default_route(), required),
        );
    }
}
#[test]
fn command_bounds_ipv4_spawn_and_io_errors_keep_protection_required() {
    for kind in [
        io::ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::BrokenPipe,
    ] {
        with_commands(
            move |_| Action::Reply(Err(kind.into())),
            || assert!(host_may_have_ipv4_default_route()),
        );
    }
}

#[test]
fn command_bounds_unknown_ipv6_address_inventory_keeps_protection_required() {
    for mode in ["slow", "bytes"] {
        with_commands(
            move |command| {
                assert_eq!(command.get_program(), "ip");
                assert_eq!(
                    arguments(command),
                    ["-6", "address", "show", "scope", "global"]
                );
                Action::Probe { mode }
            },
            || assert!(host_may_have_global_ipv6()),
        );
    }
}
#[test]
fn only_successful_empty_ipv6_address_inventory_skips_protection() {
    for (success, stdout, required) in [
        (true, &b""[..], false),
        (
            true,
            &b"2: eth0\n    inet6 fd00::1/64 scope global\n"[..],
            true,
        ),
        (false, &b""[..], true),
        (false, &b"partial"[..], true),
        (true, &b"\xff"[..], true),
        (true, &b" "[..], true),
    ] {
        with_commands(
            move |_| query_output(success, stdout),
            || assert_eq!(host_may_have_global_ipv6(), required),
        );
    }
}
