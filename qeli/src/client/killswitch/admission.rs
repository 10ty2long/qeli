//! A per-TUN chain still enforces host-wide egress policy. Two such policies do
//! not compose: the first terminal DROP blocks the other tunnel/server. Refuse a
//! conflicting ruleset before removing/rebuilding anything for the new profile.
use super::{ipt, LEGACY_CHAIN};
use std::collections::BTreeSet;

pub(super) fn check(path: &str, own_chain: &str) -> anyhow::Result<()> {
    let output = ipt(path, &["-t", "filter", "-S"])
        .map_err(|error| anyhow::anyhow!("cannot inspect {path} kill-switch ownership: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "cannot inspect {path} kill-switch ownership: {} ({})",
            String::from_utf8_lossy(&output.stderr).trim(),
            output.status
        );
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| anyhow::anyhow!("invalid UTF-8 in {path} filter inventory"))?;
    inspect(text, own_chain).map_err(|error| anyhow::anyhow!("{path}: {error}"))
}

fn inspect(text: &str, own_chain: &str) -> anyhow::Result<()> {
    let mut builtins = BTreeSet::new();
    let mut chains = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["-P", chain @ ("INPUT" | "OUTPUT" | "FORWARD"), "ACCEPT" | "DROP"] => {
                if !builtins.insert(*chain) || !chains.insert(*chain) {
                    anyhow::bail!("duplicate filter policy in kill-switch inventory");
                }
            }
            ["-N", chain] => {
                reject_other(chain, own_chain)?;
                if ["INPUT", "OUTPUT", "FORWARD"].contains(chain) || !chains.insert(*chain) {
                    anyhow::bail!("duplicate filter chain in kill-switch inventory");
                }
            }
            ["-A", chain, rule @ ..] if !rule.is_empty() => {
                reject_other(chain, own_chain)?;
                sources.insert(*chain);
            }
            _ => {
                anyhow::bail!("malformed filter inventory; cannot establish kill-switch ownership")
            }
        }
    }
    if builtins.len() != 3 || sources.iter().any(|chain| !chains.contains(chain)) {
        anyhow::bail!("incomplete filter inventory; cannot establish kill-switch ownership");
    }
    Ok(())
}

fn reject_other(chain: &str, own_chain: &str) -> anyhow::Result<()> {
    if chain == LEGACY_CHAIN || (chain.starts_with("QELI_KS_") && chain != own_chain) {
        anyhow::bail!(
            "kill-switch conflict: {chain} already exists; refusing {own_chain} without changing \
             existing firewall protection. Use one protected full-tunnel profile per network \
             namespace, or recover confirmed stale rules before retrying"
        );
    }
    Ok(())
}
