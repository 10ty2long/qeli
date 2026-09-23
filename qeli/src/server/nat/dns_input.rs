//! Process-local ownership for exact DNS INPUT permits, including failed retirement.
//! The caller serializes this registry with firewall mutations; callbacks must not re-lock it.
use std::collections::BTreeMap;

const MAX_TRACKED_RULESETS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DnsInputRules {
    pub(crate) profile: String,
    pub(crate) ipv6: bool,
    pub(crate) rules: [Vec<String>; 2],
}

impl DnsInputRules {
    pub(crate) fn new(
        profile: &str,
        tun: &str,
        pool: &str,
        listen: &str,
        port: u16,
    ) -> anyhow::Result<Self> {
        let address: std::net::IpAddr = listen.parse()?;
        Ok(Self {
            profile: profile.to_string(),
            ipv6: address.is_ipv6(),
            rules: ["udp", "tcp"]
                .map(|proto| dns_input_rule(profile, tun, pool, listen, port, proto)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DnsInputId(u64);

#[derive(Debug)]
struct Entry {
    rules: DnsInputRules,
    retired: bool,
}

#[derive(Default, Debug)]
pub(crate) struct DnsInputRegistry {
    entries: BTreeMap<DnsInputId, Entry>,
    last_id: u64,
}

impl DnsInputRegistry {
    /// Reserve ownership BEFORE the first insertion. Old failed retirement for this
    /// profile must complete before a new generation may install DNS permits.
    pub(crate) fn begin(
        &mut self,
        rules: DnsInputRules,
        cleanup: impl FnMut(&DnsInputRules) -> anyhow::Result<()>,
    ) -> anyhow::Result<DnsInputId> {
        self.retry(Some(&rules.profile), cleanup)?;
        if self.entries.values().any(|entry| entry.rules == rules) {
            anyhow::bail!(
                "DNS INPUT rules for '{}' already have a live owner",
                rules.profile
            );
        }
        if self.entries.len() >= MAX_TRACKED_RULESETS {
            anyhow::bail!("DNS INPUT ownership limit reached ({MAX_TRACKED_RULESETS}); finish pending cleanup before installing more rules");
        }
        let next = self
            .last_id
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("DNS INPUT ownership ids exhausted"))?;
        self.last_id = next;
        let id = DnsInputId(next);
        self.entries.insert(
            id,
            Entry {
                rules,
                retired: false,
            },
        );
        Ok(id)
    }

    /// Mark retired before attempting I/O. Failure (or unwind) leaves exact evidence
    /// available to retry. An old token can never retire a replacement generation.
    pub(crate) fn finish(
        &mut self,
        id: DnsInputId,
        mut cleanup: impl FnMut(&DnsInputRules) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let Some(entry) = self.entries.get_mut(&id) else {
            return Ok(());
        };
        entry.retired = true;
        cleanup(&entry.rules)?;
        self.entries.remove(&id);
        Ok(())
    }

    /// Only retired entries are eligible. A profile retry must never touch active
    /// permits or another profile; retry(None) handles every retired owner.
    pub(crate) fn retry(
        &mut self,
        profile: Option<&str>,
        mut cleanup: impl FnMut(&DnsInputRules) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let ids: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(id, entry)| {
                (entry.retired && profile.is_none_or(|profile| profile == entry.rules.profile))
                    .then_some(*id)
            })
            .collect();
        let mut errors = crate::nat_cleanup::Errors::default();
        for id in ids {
            let label = self.entries[&id].rules.profile.clone();
            errors.record(&label, self.finish(id, &mut cleanup));
        }
        errors.finish()
    }
}

pub(crate) fn dns_input_rule(
    profile: &str,
    tun: &str,
    pool_cidr: &str,
    listen: &str,
    port: u16,
    proto: &str,
) -> Vec<String> {
    vec![
        "-i".into(),
        tun.into(),
        "-s".into(),
        pool_cidr.into(),
        "-p".into(),
        proto.into(),
        "-d".into(),
        listen.into(),
        "--dport".into(),
        port.to_string(),
        "-m".into(),
        "comment".into(),
        "--comment".into(),
        format!("qeli-nat:{profile}"),
        "-j".into(),
        "ACCEPT".into(),
    ]
}

#[cfg(test)]
#[path = "dns_input/tests.rs"]
mod tests;
