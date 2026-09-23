//! In-memory delete specifications. A destination alone is not route ownership.
use super::route_command_output;

pub(super) fn route_key(args: &[String]) -> &[String] {
    let offset = usize::from(args.first().is_some_and(|arg| arg == "-6"));
    let end = offset + 3 + usize::from(args[offset + 2] == "blackhole");
    &args[..end]
}

// Kernel-equivalent host notation must not bypass another owner's reservation.
pub(super) fn same_route_key(left: &[String], right: &[String]) -> bool {
    let left = route_key(left);
    let right = route_key(right);
    left.first().is_some_and(|s| s == "-6") == right.first().is_some_and(|s| s == "-6")
        && left.iter().any(|s| s == "blackhole") == right.iter().any(|s| s == "blackhole")
        && left.last().and_then(|s| prefix(s)) == right.last().and_then(|s| prefix(s))
}

pub(super) fn delete_spec(command: &[String]) -> Vec<String> {
    let mut undo = command.to_vec();
    let action = 1 + usize::from(undo.first().is_some_and(|arg| arg == "-6"));
    undo[action] = "del".into();
    undo
}

/// Compare requested identity fields, not kernel-added defaults or display order.
/// Scope is not an IPv6 delete selector; on-link identity is checked by absence of via.
pub(super) fn route_matches_spec(spec: &[String], tokens: &[String]) -> bool {
    let key = route_key(spec);
    let blackhole = key.iter().any(|arg| arg == "blackhole");
    if tokens.first().is_some_and(|s| s == "blackhole") != blackhole {
        return false;
    }
    let destination_index = usize::from(blackhole);
    if tokens.get(destination_index).and_then(|s| prefix(s)) != key.last().and_then(|s| prefix(s)) {
        return false;
    }
    let attrs = &spec[key.len()..];
    if !blackhole
        && !attrs
            .iter()
            .any(|arg| matches!(arg.as_str(), "via" | "dev"))
    {
        return false; // Never accept an old destination-only record as proof of ownership.
    }
    let value = |name: &str| {
        tokens
            .windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| &pair[1])
    };
    for name in ["via", "dev", "src", "metric", "proto"] {
        let expected = attrs
            .windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| &pair[1]);
        if let Some(expected) = expected {
            if value(name) != Some(expected) {
                return false;
            }
        } else if name == "via" && value(name).is_some() {
            return false;
        }
    }
    true
}

fn prefix(value: &str) -> Option<(std::net::IpAddr, u8)> {
    if let Ok(net) = value.parse::<ipnet::IpNet>() {
        Some((net.network(), net.prefix_len()))
    } else {
        let ip: std::net::IpAddr = value.parse().ok()?;
        Some((ip, if ip.is_ipv4() { 32 } else { 128 }))
    }
}

fn recorded_route(spec: &[String]) -> anyhow::Result<Option<Vec<String>>> {
    let key = route_key(spec);
    let destination = key.last().expect("route destination");
    let mut query = Vec::new();
    if spec[0] == "-6" {
        query.push("-6".to_string());
    }
    query.extend([
        "route".into(),
        "show".into(),
        "exact".into(),
        destination.clone(),
    ]);
    let output = route_command_output(&query)?;
    if !output.status.success() {
        anyhow::bail!(
            "could not inspect owned route {destination}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8(output.stdout)?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let route: Option<Vec<String>> = lines
        .next()
        .map(|line| line.split_whitespace().map(str::to_string).collect());
    if lines.next().is_some() {
        anyhow::bail!("ambiguous owned route snapshot for {destination}");
    }
    if let Some(tokens) = &route {
        let index = usize::from(tokens.first().is_some_and(|s| s == "blackhole"));
        let observed = tokens.get(index).and_then(|s| prefix(s));
        if observed.is_none() || observed != prefix(destination) {
            anyhow::bail!("invalid owned route snapshot for {destination}");
        }
    }
    Ok(route)
}

/// True means the recorded route is absent; false means its identity changed.
/// A failed or lying delete never discards the entry while its matching route is visible.
pub(super) fn remove_recorded_route(spec: &[String]) -> anyhow::Result<bool> {
    let Some(current) = recorded_route(spec)? else {
        return Ok(true);
    };
    if !route_matches_spec(spec, &current) {
        return Ok(false);
    }
    let deletion = route_command_output(spec);
    match recorded_route(spec)? {
        None => Ok(true),
        Some(current) if !route_matches_spec(spec, &current) => Ok(false),
        Some(_) => {
            let detail = match deletion {
                Ok(output) if output.status.success() => {
                    "command succeeded but route remains".to_string()
                }
                Ok(output) => String::from_utf8_lossy(&output.stderr).trim().to_string(),
                Err(error) => error.to_string(),
            };
            anyhow::bail!("ip {}: {detail}", spec.join(" "))
        }
    }
}
