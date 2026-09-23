//! Explicit ownership for one Linux network-plan lifetime.
//! Entries surviving failed teardown stay reserved; a new connection cannot adopt them.
use super::ownership::{route_key, same_route_key};
#[cfg(feature = "experimental-roaming")]
use std::sync::Weak;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub(crate) struct RouteOwner(Arc<Lease>);
#[derive(Debug)]
struct Lease {
    id: u64,
    interface: String,
    generation: u64,
}
#[cfg(feature = "experimental-roaming")]
#[derive(Debug, Clone)]
pub(crate) struct RouteScope(Weak<Lease>);

struct Entry {
    id: u64,
    interface: String,
    generation: u64,
    accepting: bool,
    cleanup_failed: bool,
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
        let mut state = registry();
        if let Some(previous) = state.entries.iter().find(|e| e.interface == interface) {
            anyhow::bail!(
                "route owner for {interface} generation {} is still live or has pending cleanup",
                previous.generation
            );
        }
        state.next_id = state
            .next_id
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("route owner identity exhausted"))?;
        let id = state.next_id;
        state.entries.push(Entry {
            id,
            interface: interface.into(),
            generation,
            accepting: true,
            cleanup_failed: false,
            routes: Vec::new(),
        });
        Ok(Self(Arc::new(Lease {
            id,
            interface: interface.into(),
            generation,
        })))
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

    pub(super) fn cleanup_result(&self, failed: bool) {
        let mut state = registry();
        state
            .entries
            .iter_mut()
            .find(|e| e.id == self.0.id)
            .expect("live route owner is registered")
            .cleanup_failed = failed;
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
            if entry.routes.is_empty() && !entry.cleanup_failed {
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
        if entry.id != owner.0.id && entry.routes.iter().any(|r| same_route_key(r, spec)) {
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

#[cfg(test)]
pub(super) fn reset_tests() {
    registry().entries.clear();
}
