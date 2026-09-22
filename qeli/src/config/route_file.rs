//! Portable route-file grammar. Reading paths and applying routes remain OS operations.
use crate::transport_core::network::NumericCidr;
use std::{collections::HashSet, net::Ipv4Addr};

pub fn parse_lines(lines: &[String], source: &str, offset: u64) -> anyhow::Result<Vec<String>> {
    let mut result = vec![];
    let mut seen = HashSet::new();
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.split(['#', ';']).next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let parsed = (|| -> anyhow::Result<String> {
            let fields: Vec<_> = line.split_whitespace().collect();
            let kind = fields[0].to_ascii_lowercase();
            let candidate = if kind == "route" {
                anyhow::ensure!(fields.len() >= 2, "route has no network");
                if fields[1].contains('/') {
                    fields[1].to_string()
                } else if fields.len() >= 3 {
                    let mask = u32::from(
                        fields[2]
                            .parse::<Ipv4Addr>()
                            .map_err(|_| anyhow::anyhow!("invalid IPv4 netmask"))?,
                    );
                    let bits = mask.leading_ones();
                    anyhow::ensure!(
                        mask == if bits == 0 {
                            0
                        } else {
                            u32::MAX << (32 - bits)
                        },
                        "invalid IPv4 netmask"
                    );
                    let address = fields[1].parse::<Ipv4Addr>()?;
                    format!("{address}/{bits}")
                } else {
                    format!("{}/32", fields[1].parse::<Ipv4Addr>()?)
                }
            } else if kind == "route-ipv6" {
                anyhow::ensure!(fields.len() >= 2, "route-ipv6 has no CIDR");
                fields[1].to_string()
            } else {
                fields[0].to_string()
            };
            Ok(NumericCidr::parse(&candidate)?.render())
        })()
        .map_err(|e| {
            anyhow::anyhow!(
                "{source}:{}: {e}",
                offset.saturating_add(index as u64).saturating_add(1)
            )
        })?;
        if seen.insert(parsed.clone()) {
            result.push(parsed);
        }
    }
    Ok(result)
}
