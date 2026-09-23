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
    let expected_destination = key.last().and_then(|s| prefix(s));
    if expected_destination.is_none()
        || tokens
            .get(destination_index)
            .and_then(|s| snapshot_prefix(s, expected_destination))
            != expected_destination
    {
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
            // Linux omits RTA_PRIORITY (and therefore displayed metric) for IPv4 zero.
            if name == "metric"
                && expected == "0"
                && spec[0] != "-6"
                && !tokens.iter().any(|token| token == "metric")
            {
                continue;
            }
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

fn snapshot_prefix(
    value: &str,
    expected: Option<(std::net::IpAddr, u8)>,
) -> Option<(std::net::IpAddr, u8)> {
    if value == "default" {
        expected.filter(|(_, bits)| *bits == 0)
    } else {
        prefix(value)
    }
}

pub(super) fn recorded_route(spec: &[String]) -> anyhow::Result<Option<Vec<String>>> {
    recorded_route_with(spec, &route_command_output)
}

pub(super) fn recorded_route_with(
    spec: &[String],
    command: &dyn Fn(&[String]) -> std::io::Result<std::process::Output>,
) -> anyhow::Result<Option<Vec<String>>> {
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
    let output = command(&query)?;
    if !output.status.success() {
        anyhow::bail!(
            "could not inspect owned route {destination}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_route_snapshot(destination, &output.stdout)
}

pub(super) fn parse_route_snapshot(
    destination: &str,
    stdout: &[u8],
) -> anyhow::Result<Option<Vec<String>>> {
    let text = std::str::from_utf8(stdout)?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let route: Option<Vec<String>> = lines
        .next()
        .map(|line| line.split_whitespace().map(str::to_string).collect());
    if lines.next().is_some() {
        anyhow::bail!("ambiguous route snapshot for {destination}");
    }
    if let Some(tokens) = &route {
        let index = usize::from(tokens.first().is_some_and(|s| s == "blackhole"));
        let observed = tokens
            .get(index)
            .and_then(|s| snapshot_prefix(s, prefix(destination)));
        if observed.is_none() || observed != prefix(destination) {
            anyhow::bail!("invalid route snapshot for {destination}");
        }
    }
    Ok(route)
}

/// True means the recorded route is absent; false means its identity changed.
/// A failed or lying delete never discards the entry while its matching route is visible.
pub(super) fn remove_recorded_route_with(
    spec: &[String],
    command: &dyn Fn(&[String]) -> std::io::Result<std::process::Output>,
) -> anyhow::Result<bool> {
    let Some(current) = recorded_route_with(spec, command)? else {
        return Ok(true);
    };
    if !route_matches_spec(spec, &current) {
        return Ok(false);
    }
    let deletion = command(spec);
    match recorded_route_with(spec, command)? {
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

// A missing interface also has no routes. A failed route query alone is not proof:
// enumerate links successfully before accepting that case, including on cleanup retry.
pub(super) fn verify_interface_routes_absent(interface: &str, ipv6: bool) -> anyhow::Result<()> {
    verify_interface_routes_absent_with(interface, ipv6, &route_command_output)
}

pub(super) fn verify_interface_routes_absent_with(
    interface: &str,
    ipv6: bool,
    command: &dyn Fn(&[String]) -> std::io::Result<std::process::Output>,
) -> anyhow::Result<()> {
    let mut args = Vec::new();
    if ipv6 {
        args.push("-6".to_string());
    }
    args.extend(["route", "show", "dev", interface].map(str::to_string));
    let output = command(&args)?;
    if output.status.success() {
        if std::str::from_utf8(&output.stdout)?.trim().is_empty() {
            return Ok(());
        }
        anyhow::bail!("interface routes remain for {interface} (IPv6={ipv6})");
    }
    let links = command(&["-o", "link", "show"].map(str::to_string))?;
    if !links.status.success() {
        anyhow::bail!("could not confirm empty interface routes for {interface} (IPv6={ipv6})");
    }
    for line in std::str::from_utf8(&links.stdout)?
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let mut fields = line.split_whitespace();
        let index = fields
            .next()
            .and_then(|s| s.strip_suffix(':'))
            .and_then(|s| s.parse::<u32>().ok());
        let name = fields
            .next()
            .and_then(|s| s.strip_suffix(':'))
            .and_then(|s| s.split('@').next());
        if index.is_none_or(|index| index == 0) || name.is_none_or(str::is_empty) {
            anyhow::bail!("invalid link snapshot while checking interface {interface}");
        }
        if name == Some(interface) {
            anyhow::bail!("could not confirm empty interface routes for {interface} (IPv6={ipv6}): interface still exists");
        }
    }
    Ok(())
}
