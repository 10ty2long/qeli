//! Resource-teardown evidence and bounded joining of generation-owned TUN threads.
use crate::server_shutdown::Failures;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The wrapper reads this after dropping its guard. Sharing preserves the existing
/// RAII cleanup order without running resource teardown a second time.
#[derive(Clone, Default)]
pub(crate) struct Report(Arc<Mutex<Failures>>);
impl Report {
    pub(crate) fn record(&self, label: &str, result: anyhow::Result<()>) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record(label, result);
    }
    pub(crate) fn result(&self) -> anyhow::Result<()> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .result()
    }
}

/// A transient generation failure may be retried only after its resource cleanup succeeded.
/// Cleanup failure must reach the worker: otherwise a backoff/replacement loses old evidence.
pub(crate) struct Outcome {
    result: anyhow::Result<()>,
    restart_safe: bool,
}

impl Outcome {
    pub(crate) fn new(run: anyhow::Result<()>, cleanup: anyhow::Result<()>) -> Self {
        let restart_safe = cleanup.is_ok();
        let mut failures = Failures::default();
        failures.record("generation", run);
        failures.record("generation cleanup", cleanup);
        Self {
            result: failures.result(),
            restart_safe,
        }
    }

    pub(crate) fn can_restart(&self) -> bool {
        self.restart_safe
    }
    pub(crate) fn result(&self) -> &anyhow::Result<()> {
        &self.result
    }
    pub(crate) fn into_result(self) -> anyhow::Result<()> {
        self.result
    }
}

/// Caller raises stop flags and wakes channels first. Retain all handles during wakes
/// so pthread identifiers cannot be reused before the final wake; join only finished threads.
pub(crate) fn stop_threads(
    handles: Vec<JoinHandle<()>>,
    grace: Duration,
    mut wake: impl FnMut(),
) -> anyhow::Result<()> {
    let deadline = Instant::now() + grace;
    let mut failures = Failures::default();
    loop {
        wake();
        if handles.iter().all(JoinHandle::is_finished) {
            break;
        }
        if Instant::now() >= deadline {
            let pending = handles
                .iter()
                .filter(|handle| !handle.is_finished())
                .count();
            failures.record("TUN queues", Err(anyhow::anyhow!("{pending} queue thread(s) did not stop within {grace:?}; detached threads may retain the device")));
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    for handle in handles {
        if handle.is_finished() {
            let name = handle.thread().name().unwrap_or("unnamed").to_string();
            if let Err(panic) = handle.join() {
                let detail = panic
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("non-string panic payload");
                failures.record(
                    &format!("TUN queue thread {name}"),
                    Err(anyhow::anyhow!("panicked: {detail}")),
                );
            }
        }
    }
    failures.result()
}

#[cfg(test)]
#[path = "teardown/tests.rs"]
mod tests;
