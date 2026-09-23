//! Explicit ownership for one Linux network-plan lifetime.
//! Entries surviving failed teardown stay reserved; a new connection cannot adopt them.
use super::ownership::{recorded_route, route_key, same_route_key, verify_interface_routes_absent};
#[cfg(feature = "experimental-roaming")]
use std::sync::Weak;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub(crate) struct RouteOwner(Arc<Lease>);
#[derive(Debug)]
struct Lease {
    #[cfg(test)]
    evidence: Option<Arc<Mutex<TestEvidence>>>,
    identity_failed: std::sync::atomic::AtomicBool,
    #[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
    tun: std::sync::OnceLock<BoundTun>,
    id: u64,
    #[cfg(target_os = "linux")]
    namespace: Arc<super::identity::Namespace>,
    interface: String,
    generation: u64,
}
#[cfg(feature = "experimental-roaming")]
#[derive(Debug, Clone)]
pub(crate) struct RouteScope(Weak<Lease>);

struct Entry {
    id: u64,
    #[cfg(target_os = "linux")]
    namespace: Arc<super::identity::Namespace>,
    interface: String,
    generation: u64,
    accepting: bool,
    cleanup_failed: bool,
    interface_flushed: bool,
    orphaned: bool,
    // Reservations for unknown mutation outcomes are not delete authority.
    pending: Vec<Vec<String>>,
    routes: Vec<Vec<String>>,
}
struct Registry {
    next_id: u64,
    entries: Vec<Entry>,
}
static REGISTRY: Mutex<Registry> = Mutex::new(Registry {
    next_id: 0,
    entries: Vec::new(),
});
// Serialize check/mutate/journal within this process, including cleanup and roaming.
// This is not a kernel CAS or a command deadline.
static OPERATIONS: Mutex<()> = Mutex::new(());

fn registry() -> MutexGuard<'static, Registry> {
    REGISTRY.lock().unwrap_or_else(|e| e.into_inner())
}

impl RouteOwner {
    pub(crate) fn new(interface: &str, generation: u64) -> anyhow::Result<Self> {
        let _operation = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
        #[cfg(target_os = "linux")]
        let namespace = Arc::new(super::identity::Namespace::capture()?);
        let recovery = {
            let state = registry();
            if let Some(previous) = state.entries.iter().find(|e| e.interface == interface) {
                if !previous.orphaned || !previous.interface_flushed {
                    anyhow::bail!(
                        "route owner for {interface} generation {} is still live or has pending cleanup",
                        previous.generation
                    );
                }
                #[cfg(target_os = "linux")]
                previous.namespace.verify()?;
                Some((
                    previous.id,
                    previous
                        .routes
                        .iter()
                        .chain(&previous.pending)
                        .cloned()
                        .collect::<Vec<_>>(),
                ))
            } else {
                None
            }
        };
        if let Some((id, records)) = recovery {
            // Never adopt or delete orphan records. Only absence releases the reservation.
            // Do not hold the registry mutex while running an external query.
            for spec in records {
                if recorded_route(&spec)?.is_some() {
                    anyhow::bail!(
                        "orphan route reservation for {interface} is still present: {}",
                        spec.join(" ")
                    );
                }
            }
            for ipv6 in [false, true] {
                verify_interface_routes_absent(interface, ipv6)?;
            }
            registry().entries.retain(|e| e.id != id);
        }
        let mut state = registry();
        state.next_id = state
            .next_id
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("route owner identity exhausted"))?;
        let id = state.next_id;
        state.entries.push(Entry {
            id,
            #[cfg(target_os = "linux")]
            namespace: namespace.clone(),
            interface: interface.into(),
            generation,
            accepting: true,
            cleanup_failed: false,
            interface_flushed: false,
            orphaned: false,
            pending: Vec::new(),
            routes: Vec::new(),
        });
        Ok(Self(Arc::new(Lease {
            #[cfg(test)]
            evidence: None,
            id,
            #[cfg(target_os = "linux")]
            namespace,
            identity_failed: std::sync::atomic::AtomicBool::new(false),
            #[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
            tun: std::sync::OnceLock::new(),
            interface: interface.into(),
            generation,
        })))
    }

    #[cfg(test)]
    pub(super) fn test_new(interface: &str, generation: u64) -> anyhow::Result<Self> {
        let mut owner = Self::new(interface, generation)?;
        Arc::get_mut(&mut owner.0)
            .expect("new private lease")
            .evidence = Some(Arc::new(Mutex::new(TestEvidence {
            namespace: true,
            tunnel: true,
        })));
        Ok(owner)
    }

    #[cfg(test)]
    pub(super) fn test_evidence(&self) -> Arc<Mutex<TestEvidence>> {
        self.0
            .evidence
            .as_ref()
            .expect("synthetic route owner")
            .clone()
    }

    #[cfg(target_os = "linux")]
    pub(super) fn verify_namespace(&self) -> anyhow::Result<()> {
        self.0.namespace.verify()
    }

    // Capture only once, before any managed link/route setup. Attach mode has no
    // managed routes and never binds. No later connection can rebind this owner.
    #[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
    pub(crate) fn bind_tun(
        &self,
        tun: &Arc<crate::tun::iface::TunInterface>,
    ) -> anyhow::Result<()> {
        let _operation = self.operation()?;
        let (name, index) = tun
            .attached_link()?
            .ok_or_else(|| anyhow::anyhow!("original TUN unavailable before route setup"))?;
        if name != self.interface() {
            anyhow::bail!(
                "actual TUN name {name} differs from route owner {}; refusing setup",
                self.interface()
            );
        }
        self.0
            .tun
            .set(BoundTun {
                index,
                descriptor: Arc::downgrade(tun),
            })
            .map_err(|_| anyhow::anyhow!("route owner already bound to a TUN"))?;
        Ok(())
    }

    #[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
    pub(super) fn verify_tun(&self, tun: &crate::tun::iface::TunInterface) -> anyhow::Result<()> {
        let bound = self
            .0
            .tun
            .get()
            .ok_or_else(|| anyhow::anyhow!("route owner has no original TUN identity"))?;
        let attached = tun.attached_link()?;
        match attached {
            Some((name, current)) if name == self.interface() && current == bound.index => Ok(()),
            _ => anyhow::bail!(
                "original TUN {} was renamed, detached or changed; preserving route reservations",
                self.interface()
            ),
        }
    }

    fn observe_identity(&self, needs_tunnel: bool) -> anyhow::Result<()> {
        #[cfg(test)]
        if let Some(evidence) = &self.0.evidence {
            let evidence = evidence.lock().unwrap();
            anyhow::ensure!(evidence.namespace, "route namespace evidence unavailable");
            anyhow::ensure!(
                !needs_tunnel || evidence.tunnel,
                "original TUN evidence unavailable"
            );
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        self.verify_namespace()?;
        #[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
        if needs_tunnel {
            let bound = self
                .0
                .tun
                .get()
                .ok_or_else(|| anyhow::anyhow!("route owner has no original TUN identity"))?;
            let tun = bound
                .descriptor
                .upgrade()
                .ok_or_else(|| anyhow::anyhow!("original TUN descriptor owner has expired"))?;
            self.verify_tun(&tun)?;
        }
        // Production route adapters only exist with the Linux TUN backend. Portable
        // command tests use explicit synthetic evidence, never this fallback.
        #[cfg(not(all(target_os = "linux", any(feature = "client", feature = "server"))))]
        if needs_tunnel {
            anyhow::bail!("route owner has no TUN backend");
        }
        Ok(())
    }

    pub(super) fn identity_failed(&self) -> bool {
        self.0
            .identity_failed
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn check_identity(&self, needs_tunnel: bool) -> anyhow::Result<()> {
        if needs_tunnel && self.identity_failed() {
            anyhow::bail!("route owner previously lost identity; refusing further setup");
        }
        let result = self.observe_identity(needs_tunnel);
        if result.is_err() {
            self.0
                .identity_failed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.stop_admission();
        }
        result
    }

    pub(crate) fn verify_plan(&self) -> anyhow::Result<()> {
        self.check_identity(true)
    }

    pub(super) fn command_output(
        &self,
        args: &[String],
        needs_tunnel: bool,
    ) -> std::io::Result<std::process::Output> {
        self.check_identity(needs_tunnel)
            .map_err(std::io::Error::other)?;
        super::route_command_output(args)
    }

    pub(crate) fn interface(&self) -> &str {
        &self.0.interface
    }
    pub(crate) fn generation(&self) -> u64 {
        self.0.generation
    }

    #[cfg(feature = "experimental-roaming")]
    pub(crate) fn scope(&self) -> RouteScope {
        RouteScope(Arc::downgrade(&self.0))
    }

    pub(super) fn operation(&self) -> anyhow::Result<MutexGuard<'static, ()>> {
        let guard = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
        if !registry()
            .entries
            .iter()
            .any(|e| e.id == self.0.id && e.accepting)
        {
            anyhow::bail!("route owner is stopped; rejecting stale route operation");
        }
        self.check_identity(false)?;
        Ok(guard)
    }

    pub(super) fn cleanup_operation(&self) -> MutexGuard<'static, ()> {
        let guard = OPERATIONS.lock().unwrap_or_else(|e| e.into_inner());
        let mut state = registry();
        let entry = state
            .entries
            .iter_mut()
            .find(|e| e.id == self.0.id)
            .expect("live route owner is registered");
        // Set before the first command. Even failed cleanup must reject a late COMMIT.
        entry.accepting = false;
        guard
    }

    pub(super) fn stop_admission(&self) {
        registry()
            .entries
            .iter_mut()
            .find(|e| e.id == self.0.id)
            .expect("live route owner is registered")
            .accepting = false;
    }

    pub(super) fn cleanup_result(&self, failed: bool, interface_flushed: bool) {
        let mut state = registry();
        let entry = state
            .entries
            .iter_mut()
            .find(|e| e.id == self.0.id)
            .expect("live route owner is registered");
        entry.cleanup_failed = failed;
        entry.interface_flushed = interface_flushed;
    }
}

#[cfg(feature = "experimental-roaming")]
impl RouteScope {
    pub(crate) fn upgrade(&self) -> anyhow::Result<RouteOwner> {
        self.0.upgrade().map(RouteOwner).ok_or_else(|| {
            anyhow::anyhow!("route owner has expired; rejecting stale route operation")
        })
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut state = registry();
        if let Some(entry) = state.entries.iter_mut().find(|e| e.id == self.id) {
            entry.accepting = false;
            entry.orphaned = true;
            if entry.routes.is_empty() && entry.pending.is_empty() && !entry.cleanup_failed {
                state.entries.retain(|e| e.id != self.id);
            }
        }
    }
}

// A matching route installed by another Qeli owner is not an operator-owned route
// that we may borrow: its creator may disconnect. Refuse instead of losing its lease.
pub(super) fn ensure_unclaimed(owner: &RouteOwner, spec: &[String]) -> anyhow::Result<()> {
    let state = registry();
    for entry in &state.entries {
        if entry.id != owner.0.id
            && entry
                .routes
                .iter()
                .chain(&entry.pending)
                .any(|r| same_route_key(r, spec))
        {
            anyhow::bail!(
                "route {} belongs to another Qeli owner ({} generation {}); shared route borrowing is unsupported",
                route_key(spec).last().expect("destination"), entry.interface, entry.generation
            );
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn note_created(owner: &RouteOwner, args: &[&str]) {
    note_created_owned(owner, args.iter().map(|s| s.to_string()).collect());
}
pub(super) fn note_created_owned(owner: &RouteOwner, args: Vec<String>) {
    let mut state = registry();
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == owner.0.id)
        .expect("live route owner is registered");
    entry.routes.retain(|r| !same_route_key(r, &args));
    entry.routes.push(args);
}
#[cfg(any(test, feature = "experimental-roaming"))]
pub(super) fn recorded_undo(owner: &RouteOwner, args: &[String]) -> Option<Vec<String>> {
    registry()
        .entries
        .iter()
        .find(|e| e.id == owner.0.id)
        .and_then(|e| e.routes.iter().find(|r| same_route_key(r, args)).cloned())
}
#[cfg(all(test, feature = "experimental-roaming"))]
pub(super) fn created_by_us_owned(owner: &RouteOwner, args: &[String]) -> bool {
    recorded_undo(owner, args).is_some()
}
#[cfg(feature = "experimental-roaming")]
pub(super) fn forget_created_owned(owner: &RouteOwner, args: &[String]) {
    if let Some(entry) = registry().entries.iter_mut().find(|e| e.id == owner.0.id) {
        entry.routes.retain(|r| !same_route_key(r, args));
    }
}
pub(super) fn take_created(owner: &RouteOwner) -> Vec<Vec<String>> {
    let mut state = registry();
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == owner.0.id)
        .expect("live route owner is registered");
    std::mem::take(&mut entry.routes)
}

/// An unknown result stops admission immediately but grants no new route ownership.
pub(super) fn note_pending(owner: &RouteOwner, spec: Vec<String>) {
    let mut state = registry();
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == owner.0.id)
        .expect("live route owner is registered");
    entry.accepting = false;
    if !entry.pending.iter().any(|r| same_route_key(r, &spec)) {
        entry.pending.push(spec);
    }
}

/// Called under the operation lock after cleanup of previously proven owned records.
/// A matching pending route could have been installed by another process: do not delete it.
pub(super) fn reconcile_pending(
    owner: &RouteOwner,
    inspect: impl Fn(&[String]) -> anyhow::Result<Option<Vec<String>>>,
) -> Vec<String> {
    let records = registry()
        .entries
        .iter()
        .find(|e| e.id == owner.0.id)
        .expect("live route owner is registered")
        .pending
        .clone();
    let mut errors = Vec::new();
    for spec in records {
        match inspect(&spec) {
            Ok(None) => {
                let mut state = registry();
                let entry = state.entries.iter_mut().find(|e| e.id == owner.0.id)
                    .expect("live route owner is registered");
                entry.pending.retain(|r| !same_route_key(r, &spec));
            }
            Ok(Some(_)) => errors.push(format!(
                "unresolved route mutation; destination remains reserved without delete authority: {}",
                spec.join(" ")
            )),
            Err(error) => errors.push(format!("could not verify pending route {}: {error}", spec.join(" "))),
        }
    }
    errors
}

#[cfg(test)]
pub(super) fn reset_tests() {
    registry().entries.clear();
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct TestEvidence {
    pub(super) namespace: bool,
    pub(super) tunnel: bool,
}

// Weak evidence does not extend the TUN lifetime or keep orphaned devices alive.
#[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
#[derive(Debug)]
struct BoundTun {
    index: u32,
    descriptor: std::sync::Weak<crate::tun::iface::TunInterface>,
}
