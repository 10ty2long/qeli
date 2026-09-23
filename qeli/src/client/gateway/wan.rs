//! Read-only WAN discovery and selection of remembered cleanup targets.
use crate::system_command::Command;
use std::io;
use std::process::Output;

fn ip_output(args: &[&str]) -> io::Result<Output> {
    Command::new("ip").args(args).output()
}

pub(super) fn detect_wan() -> Option<String> {
    detect_wan_with(false, ip_output)
}

pub(super) fn detect_wan_ipv6() -> Option<String> {
    detect_wan_with(true, ip_output)
}

/// Prefer the default route itself; route-get remains the fallback for policy routing.
fn detect_wan_with(
    ipv6: bool,
    mut run: impl FnMut(&[&str]) -> io::Result<Output>,
) -> Option<String> {
    let (show, lookup): (&[&str], &[&str]) = if ipv6 {
        (
            &["-6", "route", "show", "default"],
            &["-6", "route", "get", "2606:4700:4700::1111"],
        )
    } else {
        (&["route", "show", "default"], &["route", "get", "1.1.1.1"])
    };
    if let Ok(out) = run(show) {
        if out.status.success() {
            if let Some(device) = String::from_utf8_lossy(&out.stdout)
                .lines()
                .find_map(dev_token)
            {
                return Some(device);
            }
        }
    }
    let out = run(lookup).ok()?;
    if !out.status.success() {
        return None;
    }
    dev_token(&String::from_utf8_lossy(&out.stdout))
}

fn dev_token(line: &str) -> Option<String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    tokens
        .iter()
        .position(|&token| token == "dev")
        .and_then(|index| tokens.get(index + 1))
        .map(|device| device.to_string())
}

#[cfg(test)]
#[path = "wan/tests.rs"]
mod tests;
