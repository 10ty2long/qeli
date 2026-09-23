use super::*;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

fn output(success: bool, stdout: &str) -> io::Result<Output> {
    Ok(Output {
        status: std::process::ExitStatus::from_raw(if success { 0 } else { 256 }),
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    })
}

fn expected_queries(ipv6: bool) -> [&'static [&'static str]; 2] {
    if ipv6 {
        [
            &["-6", "route", "show", "default"],
            &["-6", "route", "get", "2606:4700:4700::1111"],
        ]
    } else {
        [&["route", "show", "default"], &["route", "get", "1.1.1.1"]]
    }
}

#[test]
fn default_route_wins_without_route_get_for_each_family() {
    for ipv6 in [false, true] {
        let mut calls = 0;
        let wan = detect_wan_with(ipv6, |args| {
            calls += 1;
            assert_eq!(args, expected_queries(ipv6)[0]);
            output(
                true,
                "unreachable default\ndefault via 192.0.2.1 dev eth0 metric 100\ndefault dev eth1",
            )
        });
        assert_eq!(wan.as_deref(), Some("eth0"));
        assert_eq!(calls, 1);
    }
}

#[test]
fn failed_default_query_falls_back_for_each_family() {
    for ipv6 in [false, true] {
        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::InvalidData,
            io::ErrorKind::NotFound,
        ] {
            let mut calls = 0;
            let wan = detect_wan_with(ipv6, |args| {
                assert_eq!(args, expected_queries(ipv6)[calls]);
                calls += 1;
                if calls == 1 {
                    Err(io::Error::from(kind))
                } else {
                    output(true, "1.1.1.1 via 192.0.2.1 dev eth1 src 192.0.2.2")
                }
            });
            assert_eq!(wan.as_deref(), Some("eth1"));
            assert_eq!(calls, 2);
        }
    }
}

#[test]
fn unusable_default_output_falls_back_for_each_family() {
    for ipv6 in [false, true] {
        for (success, data) in [
            (false, "default dev rejected0"),
            (true, ""),
            (true, "unreachable default"),
            (true, "default dev"),
        ] {
            let mut calls = 0;
            let wan = detect_wan_with(ipv6, |args| {
                assert_eq!(args, expected_queries(ipv6)[calls]);
                calls += 1;
                if calls == 1 {
                    output(success, data)
                } else {
                    output(true, "destination dev eth1")
                }
            });
            assert_eq!(wan.as_deref(), Some("eth1"));
            assert_eq!(calls, 2);
        }
    }
}

#[test]
fn failed_route_get_does_not_invent_a_wan() {
    for ipv6 in [false, true] {
        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::InvalidData,
            io::ErrorKind::NotFound,
        ] {
            let mut calls = 0;
            let wan = detect_wan_with(ipv6, |args| {
                assert_eq!(args, expected_queries(ipv6)[calls]);
                calls += 1;
                if calls == 1 {
                    output(true, "")
                } else {
                    Err(io::Error::from(kind))
                }
            });
            assert_eq!(wan, None);
            assert_eq!(calls, 2);
        }
    }
}

#[test]
fn unusable_route_get_output_does_not_invent_a_wan() {
    for ipv6 in [false, true] {
        for (success, data) in [
            (false, "destination dev rejected0"),
            (true, ""),
            (true, "unreachable"),
            (true, "destination dev"),
        ] {
            let mut calls = 0;
            let wan = detect_wan_with(ipv6, |args| {
                assert_eq!(args, expected_queries(ipv6)[calls]);
                calls += 1;
                if calls == 1 {
                    output(true, "")
                } else {
                    output(success, data)
                }
            });
            assert_eq!(wan, None);
            assert_eq!(calls, 2);
        }
    }
}
