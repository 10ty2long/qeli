//! Read-only WAN discovery and selection of remembered cleanup targets.
use std::io;
use std::process::Output;

pub(super) fn detect_wan(ctx: &super::Context) -> Option<String> {
    detect_wan_with(false, |args| ctx.ip(args))
}

pub(super) fn detect_wan_ipv6(ctx: &super::Context) -> Option<String> {
    select_wan_ipv6(ctx).selected()
}

pub(super) fn select_wan_ipv6(ctx: &super::Context) -> DefaultDevice {
    detect_wan_selection_with(true, |args| ctx.ip(args))
}

/// Prefer the default route itself; route-get remains the fallback when no
/// usable default route was reported. An ambiguous best route must not fall
/// back to a single destination-specific ECMP hash choice.
fn detect_wan_with(ipv6: bool, run: impl FnMut(&[&str]) -> io::Result<Output>) -> Option<String> {
    detect_wan_selection_with(ipv6, run).selected()
}

fn detect_wan_selection_with(
    ipv6: bool,
    mut run: impl FnMut(&[&str]) -> io::Result<Output>,
) -> DefaultDevice {
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
            match preferred_default_device(&String::from_utf8_lossy(&out.stdout)) {
                DefaultDevice::Selected(device) => return DefaultDevice::Selected(device),
                DefaultDevice::Ambiguous => return DefaultDevice::Ambiguous,
                DefaultDevice::Missing => {}
            }
        }
    }
    let Ok(out) = run(lookup) else {
        return DefaultDevice::Missing;
    };
    if !out.status.success() {
        return DefaultDevice::Missing;
    }
    dev_token(&String::from_utf8_lossy(&out.stdout))
        .map_or(DefaultDevice::Missing, DefaultDevice::Selected)
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DefaultDevice {
    Missing,
    Selected(String),
    Ambiguous,
}

impl DefaultDevice {
    fn selected(self) -> Option<String> {
        match self {
            Self::Selected(device) => Some(device),
            Self::Missing | Self::Ambiguous => None,
        }
    }
}

/// `ip route show default` is not priority-ordered. A multipath route is
/// printed as a `default` header followed by indented `nexthop` lines; equal
/// metric routes can also select different devices. Both need more than one
/// WAN rule, which this name-based exit backend does not install atomically.
fn preferred_default_device(output: &str) -> DefaultDevice {
    let mut routes: Vec<(u32, Vec<String>)> = Vec::new();
    let mut current = None;
    for line in output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields.first().copied() {
            Some("default") => {
                let metric = match fields.iter().position(|field| *field == "metric") {
                    Some(index) => fields.get(index + 1).and_then(|s| s.parse::<u32>().ok()),
                    None => Some(0),
                };
                let Some(metric) = metric else {
                    current = None;
                    continue;
                };
                routes.push((metric, dev_tokens(&fields)));
                current = Some(routes.len() - 1);
            }
            Some("nexthop") => {
                if let Some(index) = current {
                    routes[index].1.extend(dev_tokens(&fields));
                }
            }
            _ => current = None,
        }
    }
    let Some(best_metric) = routes
        .iter()
        .filter(|(_, devices)| !devices.is_empty())
        .map(|(metric, _)| *metric)
        .min()
    else {
        return DefaultDevice::Missing;
    };
    let mut selected: Option<String> = None;
    for (_, devices) in routes.iter().filter(|(metric, _)| *metric == best_metric) {
        for device in devices {
            match &selected {
                Some(previous) if previous != device => return DefaultDevice::Ambiguous,
                None => selected = Some(device.clone()),
                _ => {}
            }
        }
    }
    selected.map_or(DefaultDevice::Missing, DefaultDevice::Selected)
}

fn dev_tokens(fields: &[&str]) -> Vec<String> {
    fields
        .windows(2)
        .filter(|pair| pair[0] == "dev")
        .map(|pair| pair[1].to_string())
        .collect()
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
