//! Read-only WAN discovery and selection of remembered cleanup targets.
use std::io;
use std::process::Output;

pub(super) fn detect_wan(ctx: &super::Context) -> Option<String> {
    detect_wan_with(false, |args| ctx.ip(args))
}

pub(super) fn detect_wan_ipv6(ctx: &super::Context) -> Option<String> {
    detect_wan_with(true, |args| ctx.ip(args))
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
            if let Some(device) = preferred_default_device(&String::from_utf8_lossy(&out.stdout)) {
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

/// Pick the preferred main-table default when several uplinks coexist.
/// `ip route show default` is not a priority-ordered API.
fn preferred_default_device(output: &str) -> Option<String> {
    output
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.first().copied() != Some("default") {
                return None;
            }
            let device = dev_token(line)?;
            let metric = match fields.iter().position(|field| *field == "metric") {
                Some(index) => fields.get(index + 1)?.parse::<u32>().ok()?,
                None => 0,
            };
            Some((metric, device))
        })
        .min_by_key(|(metric, _)| *metric)
        .map(|(_, device)| device)
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
