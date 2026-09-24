//! Client DNS management with lifecycle-safe per-link configuration.
//!
//! The hard requirement: *never* leave the system pointing at the tunnel
//! resolver after the tunnel is gone. New connections therefore only install
//! DNS through systemd-resolved's per-link API: deleting the tunnel link also
//! deletes its DNS state after SIGKILL, power loss, or uninstall.
//!
//! The snapshot/restore code below remains deliberately supported to recover
//! systems changed by older qeli versions. New sessions never create such a
//! snapshot or write `/etc/resolv.conf` directly.

use crate::config::client::ClientDnsConfig;
#[cfg(test)]
use crate::dns_backup::{restore_resolv, DnsBackup};
use crate::transport_core::NetworkDns;
use std::path::Path;

const RESOLV_PATH: &str = "/etc/resolv.conf";
const STATE_DIR: &str = "/var/lib/qeli";
const DNS_SETUP_BUDGET: std::time::Duration = std::time::Duration::from_secs(15);
const BACKUP_PATH: &str = "/var/lib/qeli/dns-backup.json";
pub(crate) struct DnsLease(crate::dns_lease::Lease);

fn current_scope() -> anyhow::Result<crate::dns_lease::Scope> {
    use std::os::unix::fs::MetadataExt;
    let net = std::fs::metadata("/proc/thread-self/ns/net")?;
    Ok(crate::dns_lease::Scope {
        boot: std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?
            .trim()
            .to_string(),
        device: net.dev(),
        inode: net.ino(),
    })
}

fn target(
    link: &crate::dns_lease::Link,
    tun: &crate::tun::iface::TunInterface,
) -> anyhow::Result<Option<String>> {
    let scope = current_scope()?;
    let attached = tun.attached_link()?.map(|(_, index)| index);
    Ok(crate::dns_lease::command_target(link, &scope, attached)?.map(|index| index.to_string()))
}

impl DnsLease {
    pub(crate) fn restore(&mut self, tun: &crate::tun::iface::TunInterface) -> anyhow::Result<()> {
        self.0.cleanup(|link| {
            if let Some(index) = target(link, tun)? {
                revert_resolvectl_link(&index)?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
const MARKER: &str = "# Managed by qeli VPN — original saved in /var/lib/qeli/dns-backup.json";

/// Legacy holder set written by releases that took over `/etc/resolv.conf` directly. New
/// connections use per-link systemd-resolved state and never create this file, but recovery
/// must still honour it while an older client process may be alive.
const REFCOUNT_PATH: &str = "/var/lib/qeli/dns-holders";

/// Apply exactly the resolver set already validated into the shared NetworkPlan.
/// Resolver selection and reachability routes belong to the core, for both IP families.
pub(crate) fn setup_network_plan_dns(
    config: &ClientDnsConfig,
    servers: &[NetworkDns],
    tun: &crate::tun::iface::TunInterface,
    owned: &mut Option<DnsLease>,
) -> anyhow::Result<()> {
    let until = std::time::Instant::now() + DNS_SETUP_BUDGET;
    if owned.is_some() {
        anyhow::bail!("DNS plan already owns a lease");
    }
    if config.mode != "tunnel" || servers.is_empty() {
        return Ok(());
    }
    let mut resolver_args = Vec::with_capacity(servers.len());
    for server in servers {
        let address: std::net::IpAddr = server.address.parse().map_err(|_| {
            anyhow::anyhow!("invalid network-plan DNS address '{}'", server.address)
        })?;
        if server.port == 0 {
            anyhow::bail!("invalid network-plan DNS port 0 for {address}");
        }
        resolver_args.push(if server.port == 53 {
            address.to_string()
        } else {
            format!("{address}#{}", server.port)
        });
    }

    if !resolved_is_active() {
        anyhow::bail!(
            "refusing to replace {RESOLV_PATH} with tunnel DNS: systemd-resolved is not the active \
             system resolver, so an unclean exit or uninstall could strand the host on a dead \
             resolver. Enable systemd-resolved and point {RESOLV_PATH} at its stub, or set \
             `dns = off` when NetworkManager/dnsmasq/the platform manages DNS"
        );
    }

    ensure_state_dir()?;
    let (name, index) = tun
        .attached_link()?
        .ok_or_else(|| anyhow::anyhow!("TUN disappeared before DNS setup"))?;
    let link = crate::dns_lease::Link {
        scope: current_scope()?,
        index,
        name,
    };
    let lease = crate::dns_lease::Lease::acquire(Path::new(STATE_DIR), link)?;
    // Transfer ownership BEFORE the first resolver mutation, including partial failures.
    *owned = Some(DnsLease(lease));
    let lease = owned.as_ref().expect("lease just installed");
    try_resolvectl_many(
        config,
        || {
            target(lease.0.link(), tun)?
                .ok_or_else(|| anyhow::anyhow!("TUN disappeared during DNS setup"))
        },
        &resolver_args,
        until,
    )?;
    log::info!(
        "DNS set via resolvectl on ifindex {}: {}",
        index,
        resolver_args.join(", ")
    );
    Ok(())
}

fn revert_resolvectl_link(ifname: &str) -> anyhow::Result<()> {
    let output = resolvectl_cmd()
        .args(["revert", ifname])
        .output()
        .map_err(|error| anyhow::anyhow!("cannot run resolvectl revert {ifname}: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "resolvectl revert {ifname} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Legacy resolver-file recovery is separate from per-generation link cleanup.
fn restore_legacy_dns() -> anyhow::Result<()> {
    let mut errors = Vec::new();
    // 2. Restore /etc/resolv.conf from a legacy persistent backup, but only when no older
    // client process still holds it. If the holder state cannot be locked or parsed, preserve
    // the backup and leave the host untouched rather than guessing that this process is last.
    let backup = Path::new(BACKUP_PATH);
    if backup.exists() {
        match release_dns_holder() {
            Ok(false) => {
                log::info!(
                    "DNS restore deferred: another qeli client still holds the host DNS — \
                     /etc/resolv.conf left in place"
                );
                if errors.is_empty() {
                    return Ok(());
                }
                anyhow::bail!("DNS cleanup failed: {}", errors.join("; "));
            }
            Ok(true) => {}
            Err(error) => {
                errors.push(format!(
                    "DNS restore deferred because legacy holder state is unsafe ({error}); backup kept at {BACKUP_PATH}"
                ));
                return Err(anyhow::anyhow!("DNS cleanup failed: {}", errors.join("; ")));
            }
        }
    }
    if backup.exists() {
        match crate::dns_backup::restore_and_remove(Path::new(RESOLV_PATH), backup) {
            Ok(()) => log::info!("Restored /etc/resolv.conf to its original state"),
            Err(error) => errors.push(error.to_string()),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("DNS cleanup failed: {}", errors.join("; "))
    }
}

/// Startup never reverts a live link from a durable marker alone. Only the generation
/// holding the original TUN descriptor and DNS lease may issue a resolver mutation.
pub fn recover_stale() -> anyhow::Result<()> {
    let entries = match std::fs::read_dir(STATE_DIR) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let mut errors = Vec::new();
    let mut scope = None;
    if let Some(entries) = entries {
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    errors.push(error.to_string());
                    continue;
                }
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if crate::dns_lease::is_marker(&name) {
                if scope.is_none() {
                    scope = Some(current_scope()?);
                }
                match crate::dns_lease::recover(&entry.path(),scope.as_ref().expect("scope initialized"), index_exists) {
                    Ok(crate::dns_lease::Recovery::Live) => log::warn!("DNS marker {} still names a live index; automatic revert cannot prove device ownership",entry.path().display()),
                    Ok(_) => {},
                    Err(error) => errors.push(format!("DNS marker {} retained: {error}",entry.path().display())),
                }
            } else if name.starts_with("dns-resolvectl-") {
                log::warn!("Legacy DNS marker {} retained for administrator recovery; a saved name is not ownership",entry.path().display());
            }
        }
    }
    if let Err(error) = restore_legacy_dns() {
        errors.push(error.to_string());
    }
    if !errors.is_empty() {
        anyhow::bail!("DNS recovery failed: {}", errors.join("; "));
    }
    Ok(())
}

fn index_exists(index: u32) -> anyhow::Result<bool> {
    let mut name = [0 as libc::c_char; libc::IF_NAMESIZE];
    // SAFETY: libc writes at most IF_NAMESIZE bytes to this initialized buffer.
    if !unsafe { libc::if_indextoname(index, name.as_mut_ptr()) }.is_null() {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENXIO) {
        return Ok(false);
    }
    Err(error.into())
}

// ── resolvectl ────────────────────────────────────────────────────────────

#[path = "dns/resolver_config.rs"]
mod resolver_config;

// This proves only the configured stub path. Resolver service/bus identity remains
// a separate D06 boundary; this check must not be mistaken for that proof.
fn resolved_is_active() -> bool {
    resolver_config::uses_stub(Path::new(RESOLV_PATH))
}

/// Path to the `resolvectl` binary, if it is installed at all.
///
/// The decision to use the per-link path is made by [`resolved_is_active`], which asks a
/// different and more important question. Looked up by absolute path rather than via
/// `PATH`: the client runs from a
/// systemd unit whose environment may not carry a useful `PATH`.
fn which_resolvectl() -> Option<String> {
    // An explicit override wins. It exists because the absolute-path lookup below is, by
    // design, immune to `PATH` — which also makes it immune to being pointed at a stand-in.
    // The fault-injection tests substitute a script and can only do so through the
    // environment; without this they passed on a host with no systemd-resolved (lookup
    // fails, the bare-name fallback finds the stand-in on `PATH`) and failed on one that
    // has it, where the REAL resolvectl ran and refused to configure a link named `qtest`.
    // A test that only passes where the tool is absent is worse than no test. It is also
    // the escape hatch for a distribution that keeps the binary somewhere unusual.
    if let Ok(p) = std::env::var("QELI_RESOLVECTL") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    [
        "/usr/bin/resolvectl",
        "/bin/resolvectl",
        "/usr/sbin/resolvectl",
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).exists())
    .map(str::to_string)
}

/// `resolvectl` as a runnable command, resolved to an ABSOLUTE path.
///
/// Every call site used `Command::new("resolvectl")`, which searches `PATH` — defeating the
/// whole reason [`which_resolvectl`] looks the binary up by absolute path in the first place
/// (its own doc says so: the client runs from a systemd unit whose environment may carry no
/// useful `PATH`). Where that bit, the symptom was silent: `resolvectl dns` simply failed to
/// spawn, the caller read that as "resolvectl did not work". Falls back to the bare name
/// when the binary is somewhere unusual, so
/// a working `PATH` still succeeds. (Audit 2026-07-30.)
fn resolvectl_cmd() -> crate::system_command::Command {
    crate::system_command::Command::new(
        which_resolvectl().unwrap_or_else(|| "resolvectl".to_string()),
    )
}

/// The `resolvectl domain` list for the tunnel link.
///
/// Shared with [`try_resolvectl_many`] so the decision can be tested without spawning anything —
/// it is the difference between "all DNS goes through the tunnel" and a silent split.
fn routing_domains(config: &ClientDnsConfig) -> Vec<String> {
    let mut domains: Vec<String> = config.search_domains.clone();
    if config.redirect_all || config.mode.eq_ignore_ascii_case("tunnel") {
        domains.push("~.".to_string());
    }
    domains
}

#[cfg(test)]
fn try_resolvectl(config: &ClientDnsConfig, ifname: &str, dns_addr: &str) -> bool {
    try_resolvectl_many(
        config,
        || Ok(ifname.to_string()),
        &[dns_addr.to_string()],
        std::time::Instant::now() + DNS_SETUP_BUDGET,
    )
    .is_ok()
}

fn try_resolvectl_many(
    config: &ClientDnsConfig,
    mut current_target: impl FnMut() -> anyhow::Result<String>,
    dns_addrs: &[String],
    until: std::time::Instant,
) -> anyhow::Result<()> {
    let domains = routing_domains(config);
    for (operation, values) in [("dns", dns_addrs), ("domain", domains.as_slice())] {
        if values.is_empty() {
            continue;
        }
        if std::time::Instant::now() >= until {
            anyhow::bail!(
                "DNS setup command budget exhausted; generation rollback retains its DNS lease"
            );
        }
        // Recheck the original descriptor before each mutation; use its captured numeric
        // index so a rename does not redirect a later operation to a same-name replacement.
        let index = current_target()?;
        let output = resolvectl_cmd()
            .args([operation, &index])
            .args(values)
            .output_until(until)?;
        if !output.status.success() {
            anyhow::bail!("resolvectl {operation} on {index} failed with {}: {}; generation rollback retains its DNS lease",output.status,String::from_utf8_lossy(&output.stderr).trim());
        }
    }
    Ok(())
}

// ── /etc/resolv.conf capture & restore (pure file logic, path-injectable) ───

fn ensure_state_dir() -> anyhow::Result<()> {
    std::fs::create_dir_all(STATE_DIR)
        .map_err(|e| anyhow::anyhow!("cannot create state dir {}: {}", STATE_DIR, e))
}

/// Live-holder set for the host DNS takeover: one line per still-running client pid.
/// Read under a lock, filtered to pids that are actually alive (so a SIGKILLed instance
/// does not pin the takeover forever), and returned.
fn read_live_holders() -> anyhow::Result<Vec<u32>> {
    let text = match std::fs::read_to_string(REFCOUNT_PATH) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => anyhow::bail!("cannot read {REFCOUNT_PATH}: {error}"),
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim().parse::<u32>().map_err(|error| {
                anyhow::anyhow!("invalid PID in {REFCOUNT_PATH} ({line:?}): {error}")
            })
        })
        .filter_map(|pid| match pid {
            Ok(pid) if pid_alive(pid) => Some(Ok(pid)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn write_holders(pids: &[u32]) -> anyhow::Result<()> {
    let body: String = pids.iter().map(|p| format!("{p}\n")).collect();
    crate::util::write_atomic_private(REFCOUNT_PATH, body.as_bytes())
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    // kill(pid, 0): 0 or EPERM => the process exists; ESRCH => it does not.
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    rc == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    true
}

/// Release: returns (remaining, last).
fn compute_release(holders: Vec<u32>, me: u32) -> (Vec<u32>, bool) {
    let remaining: Vec<u32> = holders.into_iter().filter(|&p| p != me).collect();
    let last = remaining.is_empty();
    (remaining, last)
}

/// Drop this process from the holder set. Returns true when it was the LAST holder — the
/// only case in which the caller should restore the original and delete the backup.
fn release_dns_holder() -> anyhow::Result<bool> {
    ensure_state_dir()?;
    let _lock = crate::util::FileLock::acquire(REFCOUNT_PATH)?;
    let (remaining, last) = compute_release(read_live_holders()?, std::process::id());
    if last {
        match std::fs::remove_file(REFCOUNT_PATH) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => anyhow::bail!("cannot remove {REFCOUNT_PATH}: {error}"),
        }
    } else {
        write_holders(&remaining)?;
    }
    Ok(last)
}

/// Capture the current resolv.conf state into `backup`, exactly once.
///
/// Idempotent: if `backup` already exists we keep the previously-saved
/// original. If the current file is already ours (contains `marker`) but no
/// backup exists, we record `managed-no-original` so restore falls back to a
/// working public resolver rather than leaving a dangling tunnel address.
#[cfg(test)]
fn capture_original(resolv: &Path, backup: &Path, marker: &str) -> anyhow::Result<()> {
    if backup.exists() {
        return Ok(());
    }

    let snapshot = match std::fs::symlink_metadata(resolv) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let target = std::fs::read_link(resolv)
                .map_err(|e| anyhow::anyhow!("read_link {}: {}", resolv.display(), e))?;
            DnsBackup {
                kind: "symlink".into(),
                target: Some(target.to_string_lossy().into_owned()),
                content: None,
                mode: None,
            }
        }
        Ok(_meta) => {
            let content = std::fs::read_to_string(resolv).unwrap_or_default();
            if content.contains(marker) {
                // Our own file with no saved original — corrupted prior state.
                DnsBackup {
                    kind: "managed-no-original".into(),
                    target: None,
                    content: None,
                    mode: None,
                }
            } else {
                #[cfg(unix)]
                let mode = {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::metadata(resolv)
                        .ok()
                        .map(|m| m.permissions().mode())
                };
                #[cfg(not(unix))]
                let mode = None;
                DnsBackup {
                    kind: "file".into(),
                    target: None,
                    content: Some(content),
                    mode,
                }
            }
        }
        Err(_) => DnsBackup {
            kind: "absent".into(),
            target: None,
            content: None,
            mode: None,
        },
    };

    let json = serde_json::to_string(&snapshot)?;
    write_atomic(backup, json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
fn write_managed_resolv(
    resolv: &Path,
    dns_server: &str,
    search: &[String],
    marker: &str,
) -> anyhow::Result<()> {
    write_managed_resolv_many(resolv, &[dns_server.to_string()], search, marker)
}

#[cfg(test)]
fn write_managed_resolv_many(
    resolv: &Path,
    dns_servers: &[String],
    search: &[String],
    marker: &str,
) -> anyhow::Result<()> {
    let mut content = String::new();
    content.push_str(marker);
    content.push('\n');
    for dns_server in dns_servers {
        content.push_str(&format!("nameserver {}\n", dns_server));
    }
    if !search.is_empty() {
        content.push_str(&format!("search {}\n", search.join(" ")));
    }
    write_atomic(resolv, content.as_bytes())
}

/// Write a file atomically (tmp in the same dir, then rename). Thin wrapper over
/// [`crate::util::write_atomic`] — the single shared implementation (also used by
/// the server's config/users/key writes), which on Unix uses `O_EXCL` +
/// `O_NOFOLLOW` against symlink pre-planting (H-5) and preserves the target's
/// mode. Replacing a symlink with the renamed regular file is intentional —
/// `restore_resolv` recreates the link from the backup.
#[cfg(test)]
fn write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    crate::util::write_atomic(path, bytes)
}

#[cfg(test)]
mod tests {

    /// A default client (`dns.mode = tunnel`) MUST claim the `~.` catch-all routing domain.
    ///
    /// It used to be gated on `redirect_all`, which no INI key can set, so the catch-all was
    /// never emitted: systemd-resolved kept splitting queries between the tunnel resolver and
    /// the physical link's, and the log still said DNS was set. Deserialized from an EMPTY
    /// document on purpose — `#[derive(Default)]` ignores `#[serde(default = "…")]`, so
    /// testing `::default()` would not exercise the real default. (Audit 2026-08-04, H-01.)
    #[test]
    fn tunnel_dns_mode_claims_the_catch_all_routing_domain() {
        let dns: crate::config::client::ClientDnsConfig =
            serde_json::from_str("{}").expect("empty document uses the serde defaults");
        assert_eq!(dns.mode, "tunnel", "the shipped default");
        assert!(
            !dns.redirect_all,
            "no INI key sets this; it must not be required"
        );
        assert!(
            super::routing_domains(&dns).contains(&"~.".to_string()),
            "dns.mode = tunnel must send ALL queries through the tunnel link"
        );

        // `dns = off` / `system` means the user keeps their own resolver — do not hijack it.
        let mut off = dns.clone();
        off.mode = "off".to_string();
        assert!(
            !super::routing_domains(&off).contains(&"~.".to_string()),
            "dns = off must leave the host resolver alone"
        );

        // Search domains still ride along, and `redirect_all` alone still works.
        let mut with_search = off.clone();
        with_search.search_domains = vec!["corp.example".to_string()];
        assert_eq!(super::routing_domains(&with_search), vec!["corp.example"]);
        with_search.redirect_all = true;
        assert_eq!(
            super::routing_domains(&with_search),
            vec!["corp.example", "~."]
        );
    }

    /// An unconfigured client must NOT silently send its DNS to a third party.
    ///
    /// Core DNS selection consults `servers` then the push and `fallback_servers`, so
    /// a non-empty DEFAULT for either one makes it unreachable — which is exactly how a
    /// `["1.1.1.1", "8.8.8.8"]` default cancelled the R5 fix and sent every query of every
    /// default-configured client to Cloudflare. Deserialized from an EMPTY document on purpose:
    /// `#[derive(Default)]` ignores `#[serde(default = "…")]`, so testing `::default()` would
    /// pass no matter what the serde default said. (Audit 2026-07-30, #8.)
    #[test]
    fn an_unconfigured_client_has_no_third_party_resolver() {
        let dns: crate::config::client::ClientDnsConfig =
            serde_json::from_str("{}").expect("empty config deserializes");
        assert_eq!(
            dns.mode, "tunnel",
            "guard: this test assumes tunnel is the default mode"
        );
        assert!(
            dns.servers.is_empty(),
            "dns.servers must not default to anything"
        );
        assert!(
            dns.fallback_servers.is_empty(),
            "dns.fallback_servers must not default to a third-party resolver — that silently              overrides the user's choice and makes the 'no resolver configured' refusal dead code"
        );
    }

    /// Legacy recovery must remove only this process and defer restoration while another
    /// live holder remains.
    #[test]
    fn legacy_holder_release_restores_only_for_the_last_process() {
        let (holders, last) = super::compute_release(vec![100, 200], 100);
        assert!(
            !last,
            "the first to leave must NOT restore while another holds DNS"
        );
        assert_eq!(holders, vec![200]);

        let (holders, last) = super::compute_release(holders, 200);
        assert!(last, "the last holder out restores the original");
        assert!(holders.is_empty());
    }

    use super::*;
    use std::path::PathBuf;

    /// Unique temp workspace per test.
    pub(super) struct Tmp(PathBuf);
    impl Tmp {
        pub(super) fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!(
                "qeli-dns-{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&p).unwrap();
            Tmp(p)
        }
        pub(super) fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn read(p: &Path) -> String {
        std::fs::read_to_string(p).unwrap()
    }

    #[test]
    fn capture_and_restore_regular_file() {
        let t = Tmp::new("file");
        let resolv = t.path("resolv.conf");
        let backup = t.path("backup.json");
        std::fs::write(&resolv, "nameserver 192.168.1.1\n").unwrap();

        capture_original(&resolv, &backup, MARKER).unwrap();
        write_managed_resolv(&resolv, "10.0.0.1", &[], MARKER).unwrap();
        assert!(read(&resolv).contains("10.0.0.1"));
        assert!(read(&resolv).contains(MARKER));

        restore_resolv(&resolv, &backup).unwrap();
        assert_eq!(read(&resolv), "nameserver 192.168.1.1\n");
    }

    #[test]
    fn capture_is_idempotent_across_reconnects() {
        // The core bug: a second setup must NOT overwrite the saved original
        // with our generated file.
        let t = Tmp::new("reconnect");
        let resolv = t.path("resolv.conf");
        let backup = t.path("backup.json");
        std::fs::write(&resolv, "nameserver 9.9.9.9\n").unwrap();

        capture_original(&resolv, &backup, MARKER).unwrap();
        write_managed_resolv(&resolv, "10.0.0.1", &[], MARKER).unwrap();
        // Reconnect: setup runs again while resolv.conf is already ours.
        capture_original(&resolv, &backup, MARKER).unwrap();

        restore_resolv(&resolv, &backup).unwrap();
        assert_eq!(
            read(&resolv),
            "nameserver 9.9.9.9\n",
            "original must survive reconnect"
        );
    }

    #[test]
    #[cfg(unix)]
    fn capture_and_restore_symlink() {
        let t = Tmp::new("symlink");
        let resolv = t.path("resolv.conf");
        let real = t.path("stub-resolv.conf");
        std::fs::write(&real, "nameserver 127.0.0.53\n").unwrap();
        std::os::unix::fs::symlink(&real, &resolv).unwrap();

        capture_original(&resolv, &t.path("backup.json"), MARKER).unwrap();
        write_managed_resolv(&resolv, "10.0.0.1", &[], MARKER).unwrap();
        // Our write replaced the symlink with a regular file.
        assert!(!std::fs::symlink_metadata(&resolv)
            .unwrap()
            .file_type()
            .is_symlink());

        restore_resolv(&resolv, &t.path("backup.json")).unwrap();
        let meta = std::fs::symlink_metadata(&resolv).unwrap();
        assert!(meta.file_type().is_symlink(), "symlink must be recreated");
        assert_eq!(std::fs::read_link(&resolv).unwrap(), real);
    }

    #[test]
    fn absent_original_is_removed_on_restore() {
        let t = Tmp::new("absent");
        let resolv = t.path("resolv.conf");
        let backup = t.path("backup.json");
        // No resolv.conf exists yet.
        capture_original(&resolv, &backup, MARKER).unwrap();
        write_managed_resolv(&resolv, "10.0.0.1", &[], MARKER).unwrap();
        assert!(resolv.exists());

        restore_resolv(&resolv, &backup).unwrap();
        assert!(
            !resolv.exists(),
            "file we created must be removed when there was no original"
        );
    }

    #[test]
    fn managed_file_without_backup_restores_to_public_resolver() {
        // Simulates a crashed prior run: resolv.conf is ours, backup is gone.
        let t = Tmp::new("orphan");
        let resolv = t.path("resolv.conf");
        let backup = t.path("backup.json");
        write_managed_resolv(&resolv, "10.0.0.1", &[], MARKER).unwrap();

        capture_original(&resolv, &backup, MARKER).unwrap();
        let snap: DnsBackup = serde_json::from_str(&read(&backup)).unwrap();
        assert_eq!(snap.kind, "managed-no-original");

        restore_resolv(&resolv, &backup).unwrap();
        let restored = read(&resolv);
        assert!(
            restored.contains("1.1.1.1"),
            "must leave a working resolver, not the dead tunnel IP"
        );
        assert!(!restored.contains("10.0.0.1"));
    }
}

// ── fault injection: a PARTIAL resolvectl failure must not read as success ───
//
// `resolvectl dns` and `resolvectl domain` are two calls, and only the pair does what
// the mode promises: the server address decides WHERE queries go, the routing domains
// decide WHICH queries take that link. With `~.` the domains are the difference between
// "all DNS goes through the tunnel" and "almost none does" — so a failure of the second
// call while the first succeeded is a silent DNS leak, reported as a working tunnel.
//
// Only reproducible by making the command fail on demand, hence the stub behind QELI_RESOLVECTL.
#[cfg(all(test, target_os = "linux"))]
mod fault_injection {
    use super::*;
    use std::io::Write;
    use std::sync::{Mutex, MutexGuard};

    static SERIAL: Mutex<()> = Mutex::new(());

    struct Resolvectl {
        dir: std::path::PathBuf,
        _guard: MutexGuard<'static, ()>,
        old_path: String,
    }

    impl Resolvectl {
        fn new(tag: &str, fail_on: &[&str]) -> Resolvectl {
            let guard = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
            let dir = std::env::temp_dir().join(format!("qeli-rslv-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let log = dir.join("calls.log");

            let mut script = String::from("#!/bin/sh\n");
            script.push_str(&format!("echo \"$@\" >> {}\n", log.display()));
            script.push_str("case \"$*\" in\n");
            for cond in fail_on {
                script.push_str(&format!("  *\"{cond}\"*) exit 1;;\n"));
            }
            script.push_str("esac\nexit 0\n");

            let bin = dir.join("resolvectl");
            let mut f = std::fs::File::create(&bin).unwrap();
            f.write_all(script.as_bytes()).unwrap();
            drop(f);
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

            // Point the code at the stand-in explicitly. Prepending to PATH is not enough:
            // the lookup is deliberately immune to PATH, so on a host that HAS a real
            // resolvectl the tests used to drive the real one against a link named `qtest`.
            let old_path = std::env::var("QELI_RESOLVECTL").unwrap_or_default();
            std::env::set_var("QELI_RESOLVECTL", bin.as_os_str());
            Resolvectl {
                dir,
                _guard: guard,
                old_path,
            }
        }

        fn calls(&self) -> String {
            std::fs::read_to_string(self.dir.join("calls.log")).unwrap_or_default()
        }
    }

    impl Drop for Resolvectl {
        fn drop(&mut self) {
            if self.old_path.is_empty() {
                std::env::remove_var("QELI_RESOLVECTL");
            } else {
                std::env::set_var("QELI_RESOLVECTL", &self.old_path);
            }
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// Full-tunnel DNS: the `~.` catch-all is what makes every query take the link.
    fn redirect_all() -> ClientDnsConfig {
        ClientDnsConfig {
            redirect_all: true,
            ..Default::default()
        }
    }

    #[test]
    fn a_refused_routing_domain_is_not_reported_as_success() {
        // `dns` lands, `domain` does not. The old code discarded the second result and
        // returned true, so the caller logged "DNS set" and never fell back — while every
        // query kept going to the physical resolver.
        let rc = Resolvectl::new("domain", &["domain qtest"]);
        assert!(
            !try_resolvectl(&redirect_all(), "qtest", "10.0.0.1"),
            "a failed routing-domain call must report failure so the caller refuses takeover"
        );
        assert!(
            !rc.calls().contains("revert qtest"),
            "only the owning generation guard may revert partial state:\n{}",
            rc.calls()
        );
    }

    #[test]
    fn a_working_resolvectl_reports_success_and_sets_both() {
        let rc = Resolvectl::new("ok", &[]);
        assert!(try_resolvectl(&redirect_all(), "qtest", "10.0.0.1"));
        let calls = rc.calls();
        assert!(
            calls.contains("dns qtest 10.0.0.1") && calls.contains("domain qtest"),
            "both halves must be applied:\n{calls}"
        );
        assert!(
            !calls.contains("revert"),
            "nothing to revert on the success path:\n{calls}"
        );
    }

    #[test]
    fn a_refused_dns_call_fails_before_touching_domains() {
        // The first call failing must also revert any partial per-link state and make the
        // caller refuse a persistent resolv.conf takeover.
        let rc = Resolvectl::new("dns", &["dns qtest"]);
        assert!(!try_resolvectl(&redirect_all(), "qtest", "10.0.0.1"));
        assert!(
            !rc.calls().contains("domain qtest"),
            "no point setting routing domains on a link whose server was refused:\n{}",
            rc.calls()
        );
        assert!(
            !rc.calls().contains("revert qtest"),
            "command failure must leave rollback to the generation guard:\n{}",
            rc.calls()
        );
    }
    #[test]
    fn identity_change_between_dns_and_domain_stops_further_commands() {
        let rc = Resolvectl::new("identity", &[]);
        let mut calls = 0;
        let result = try_resolvectl_many(
            &redirect_all(),
            || {
                calls += 1;
                if calls == 2 {
                    anyhow::bail!("original TUN disappeared");
                }
                Ok("42".to_string())
            },
            &["10.0.0.1".to_string()],
            std::time::Instant::now() + DNS_SETUP_BUDGET,
        );
        assert!(result.is_err());
        assert_eq!(rc.calls().trim(), "dns 42 10.0.0.1");
    }

    #[test]
    fn dns_and_domain_share_deadline_and_partial_lease_survives() {
        let rc = Resolvectl::new("shared-budget", &[]);
        let script = format!(
            "#!/bin/sh\necho \"$@\" >> '{}'\ncase \"$1\" in\n dns) sleep 0.15;;\n domain) sleep 0.70;;\nesac\nexit 0\n",
            rc.dir.join("calls.log").display()
        );
        std::fs::write(rc.dir.join("resolvectl"), script).unwrap();
        let mut lease = crate::dns_lease::Lease::acquire(
            &rc.dir,
            crate::dns_lease::Link {
                scope: crate::dns_lease::Scope {
                    boot: "00000000-0000-0000-0000-000000000001".into(),
                    device: 4,
                    inode: 42,
                },
                index: 42,
                name: "qtest".into(),
            },
        )
        .unwrap();
        let marker = std::fs::read_dir(&rc.dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| crate::dns_lease::is_marker(p.file_name().unwrap().to_str().unwrap()))
            .unwrap();
        let original = std::fs::read(&marker).unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_millis(600);
        let result = try_resolvectl_many(
            &redirect_all(),
            || Ok("42".into()),
            &["10.0.0.1".into()],
            until,
        );
        assert!(
            result.is_err(),
            "two commands must not receive independent budgets"
        );
        assert!(rc.calls().contains("dns 42 10.0.0.1"));
        assert!(rc.calls().contains("domain 42 ~."));
        assert!(!rc.calls().contains("revert"));
        assert_eq!(std::fs::read(&marker).unwrap(), original);
        // Rollback is a separate owned operation with its own command deadline.
        lease.cleanup(|_| revert_resolvectl_link("42")).unwrap();
        assert!(rc.calls().contains("revert 42"));
        assert!(!marker.exists());
    }

    #[test]
    fn expired_dns_budget_starts_no_target_probe_or_command() {
        let rc = Resolvectl::new("expired-budget", &[]);
        let result = try_resolvectl_many(
            &redirect_all(),
            || panic!("expired operation must not probe the target"),
            &["10.0.0.1".into()],
            std::time::Instant::now(),
        );
        assert!(result.unwrap_err().to_string().contains("budget exhausted"));
        assert!(rc.calls().is_empty());
    }

    #[test]
    fn slow_target_probe_cannot_start_domain_after_shared_deadline() {
        let rc = Resolvectl::new("target-budget", &[]);
        let until = std::time::Instant::now() + std::time::Duration::from_millis(250);
        let mut probes = 0;
        let result = try_resolvectl_many(
            &redirect_all(),
            || {
                probes += 1;
                if probes == 2 {
                    std::thread::sleep(
                        until.saturating_duration_since(std::time::Instant::now())
                            + std::time::Duration::from_millis(10),
                    );
                }
                Ok("42".into())
            },
            &["10.0.0.1".into()],
            until,
        );
        assert!(result.is_err());
        assert_eq!(probes, 2);
        assert_eq!(rc.calls().trim(), "dns 42 10.0.0.1");
    }
}
