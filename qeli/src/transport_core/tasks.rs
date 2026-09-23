//! One connection generation owns both its producers and their child tasks.
use std::future::{poll_fn, Future};
use std::sync::{Arc, Mutex, Weak};
use tokio::task::JoinSet;

#[derive(Default)]
struct State {
    closed: bool,
    tasks: JoinSet<()>,
}

#[derive(Default)]
pub(crate) struct TaskGroup(Arc<Mutex<State>>);

// A producer must not keep its own owner alive through the task registry.
#[derive(Clone)]
pub(crate) struct Spawner(Weak<Mutex<State>>);

fn report(result: Result<(), tokio::task::JoinError>) {
    if result.is_err_and(|error| !error.is_cancelled()) {
        // Do not copy a possibly sensitive panic payload into connection diagnostics.
        log::error!("connection background task failed");
    }
}

impl TaskGroup {
    pub(crate) fn spawner(&self) -> Spawner {
        Spawner(Arc::downgrade(&self.0))
    }

    fn close(&self) {
        let mut state = crate::util::lock_or_recover(&self.0, "connection_tasks");
        state.closed = true;
        state.tasks.abort_all();
    }

    pub(crate) async fn finish(&mut self) {
        self.close();
        // Keep handles in the group while pending: cancelling this waiter cannot detach
        // in-flight work. A retry waits for the same tasks, including blocking closures.
        while let Some(result) = poll_fn(|cx| {
            crate::util::lock_or_recover(&self.0, "connection_tasks")
                .tasks
                .poll_join_next(cx)
        })
        .await
        {
            report(result);
        }
    }
}

impl Drop for TaskGroup {
    fn drop(&mut self) {
        // Drop cannot join asynchronously or interrupt a running blocking closure. It does
        // close admission and request cancellation even if producers still hold Spawners.
        self.close();
    }
}

impl Spawner {
    fn admit(&self, spawn: impl FnOnce(&mut JoinSet<()>)) -> bool {
        let Some(shared) = self.0.upgrade() else {
            return false;
        };
        let mut state = crate::util::lock_or_recover(&shared, "connection_tasks");
        if state.closed {
            // Drop rejected captures outside the mutex: their destructors may use a Spawner.
            drop(state);
            return false;
        }
        // Long-lived handovers must not accumulate completed task allocations.
        let mut completed = Vec::new();
        while let Some(result) = state.tasks.try_join_next() {
            completed.push(result);
        }
        // Admission and actual spawning share the shutdown lock. No already-spawned handle
        // can arrive after the drain, and Tokio does not synchronously poll the new future.
        spawn(&mut state.tasks);
        drop(state);
        for result in completed {
            report(result);
        }
        true
    }

    pub(crate) fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) -> bool {
        self.admit(|tasks| {
            tasks.spawn(future);
        })
    }

    #[cfg(any(test, all(target_os = "linux", feature = "experimental-roaming")))]
    pub(crate) async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (send, receive) = tokio::sync::oneshot::channel();
        if !self.admit(|tasks| {
            tasks.spawn_blocking(move || {
                let _ = send.send(work());
            });
        }) {
            return None;
        }
        receive.await.ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    const DEADLINE: Duration = Duration::from_secs(5);
    const PROBE: Duration = Duration::from_millis(30);

    struct Lease(Option<tokio::sync::oneshot::Sender<()>>);
    impl Drop for Lease {
        fn drop(&mut self) {
            if let Some(send) = self.0.take() {
                let _ = send.send(());
            }
        }
    }

    fn pending_lease(group: &TaskGroup) -> tokio::sync::oneshot::Receiver<()> {
        let (send, receive) = tokio::sync::oneshot::channel();
        let lease = Lease(Some(send));
        assert!(group.spawner().spawn(async move {
            let _lease = lease;
            std::future::pending::<()>().await;
        }));
        receive
    }

    async fn await_release(receive: tokio::sync::oneshot::Receiver<()>) {
        tokio::time::timeout(DEADLINE, receive)
            .await
            .unwrap()
            .unwrap();
    }

    async fn blocked_producer(
        group: &TaskGroup,
    ) -> (
        std::sync::mpsc::Sender<()>,
        tokio::sync::oneshot::Receiver<bool>,
    ) {
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        let (result, rejected) = tokio::sync::oneshot::channel();
        let spawner = group.spawner();
        assert!(group.spawner().spawn(async move {
            started.send(()).unwrap();
            let _ = blocked.recv();
            // Abort cannot interrupt this synchronous section, but admission must be sealed.
            let admitted = spawner.spawn(async { panic!("obsolete child must never run") });
            let _ = result.send(admitted);
        }));
        await_release(ready).await;
        (release, rejected)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_joins_producer_and_rejects_its_late_child() {
        let mut group = TaskGroup::default();
        let (release, admitted) = blocked_producer(&group).await;
        let mut finishing = tokio::spawn(async move {
            group.finish().await;
            assert!(crate::util::lock_or_recover(&group.0, "test")
                .tasks
                .is_empty());
        });
        assert!(tokio::time::timeout(PROBE, &mut finishing).await.is_err());
        release.send(()).unwrap();
        tokio::time::timeout(DEADLINE, finishing)
            .await
            .unwrap()
            .unwrap();
        assert!(!admitted.await.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_shutdown_retains_handles_for_retry() {
        let mut group = TaskGroup::default();
        let (release, admitted) = blocked_producer(&group).await;
        assert!(tokio::time::timeout(PROBE, group.finish()).await.is_err());
        assert!(!crate::util::lock_or_recover(&group.0, "test")
            .tasks
            .is_empty());
        release.send(()).unwrap();
        tokio::time::timeout(DEADLINE, group.finish())
            .await
            .unwrap();
        assert!(!admitted.await.unwrap());
        group.finish().await; // Idempotent after drain; admission stays closed.
        assert!(!group.spawner().spawn(async {}));
    }

    #[tokio::test]
    async fn owner_drop_aborts_children_despite_surviving_spawner() {
        let group = TaskGroup::default();
        let spawner = group.spawner();
        let released = pending_lease(&group);
        drop(group);
        assert!(!spawner.spawn(async { panic!("dropped generation") }));
        await_release(released).await;
        assert!(spawner.0.upgrade().is_none());
    }

    #[tokio::test]
    async fn early_error_and_unwind_abort_owned_children() {
        async fn fail(
            receive: &mut Option<tokio::sync::oneshot::Receiver<()>>,
        ) -> anyhow::Result<()> {
            let group = TaskGroup::default();
            *receive = Some(pending_lease(&group));
            Err(anyhow::anyhow!("setup rejected"))?;
            Ok(())
        }
        let mut released = None;
        assert!(fail(&mut released).await.is_err());
        await_release(released.take().unwrap()).await;
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let group = TaskGroup::default();
            released = Some(pending_lease(&group));
            panic!("fixture unwind");
        }));
        assert!(unwind.is_err());
        await_release(released.unwrap()).await;
    }

    #[tokio::test]
    async fn replacement_reaps_completed_handles_and_keeps_active_tasks() {
        let mut group = TaskGroup::default();
        let (send, receive) = tokio::sync::oneshot::channel();
        group.spawner().spawn(async move {
            send.send(()).unwrap();
        });
        receive.await.unwrap();
        let first = pending_lease(&group);
        assert_eq!(
            crate::util::lock_or_recover(&group.0, "test").tasks.len(),
            1
        );
        let second = pending_lease(&group);
        assert_eq!(
            crate::util::lock_or_recover(&group.0, "test").tasks.len(),
            2
        );
        group.finish().await;
        await_release(first).await;
        await_release(second).await;
    }

    #[tokio::test]
    async fn panic_does_not_skip_other_destructors() {
        let mut group = TaskGroup::default();
        let (started, ready) = tokio::sync::oneshot::channel();
        group.spawner().spawn(async move {
            started.send(()).unwrap();
            panic!("fixture panic");
        });
        let released = pending_lease(&group);
        ready.await.unwrap();
        group.finish().await;
        await_release(released).await;
    }

    #[tokio::test]
    async fn blocking_worker_is_joined_even_after_its_async_waiter_is_aborted() {
        let mut group = TaskGroup::default();
        let spawner = group.spawner();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        let complete = Arc::new(AtomicBool::new(false));
        let completed = complete.clone();
        group.spawner().spawn(async move {
            spawner
                .blocking(move || {
                    started.send(()).unwrap();
                    let _ = blocked.recv();
                    completed.store(true, Ordering::Release);
                })
                .await;
        });
        await_release(ready).await;
        assert!(tokio::time::timeout(PROBE, group.finish()).await.is_err());
        assert!(!complete.load(Ordering::Acquire));
        release.send(()).unwrap();
        tokio::time::timeout(DEADLINE, group.finish())
            .await
            .unwrap();
        assert!(
            complete.load(Ordering::Acquire),
            "network cleanup must follow the worker"
        );
    }

    #[tokio::test]
    async fn closed_group_rejects_blocking_work_and_drops_its_captures() {
        let mut group = TaskGroup::default();
        group.finish().await;
        let (send, receive) = tokio::sync::oneshot::channel();
        let lease = Lease(Some(send));
        assert_eq!(
            group
                .spawner()
                .blocking(move || {
                    let _lease = lease;
                    panic!("obsolete blocking work must never run");
                })
                .await,
            None::<()>
        );
        await_release(receive).await;
    }

    #[tokio::test]
    async fn rejected_future_can_use_spawner_in_its_destructor() {
        struct Reentrant(Spawner, Arc<AtomicBool>);
        impl Drop for Reentrant {
            fn drop(&mut self) {
                assert!(!self.0.spawn(async {}));
                self.1.store(true, Ordering::Release);
            }
        }
        let mut group = TaskGroup::default();
        group.finish().await;
        let dropped = Arc::new(AtomicBool::new(false));
        let capture = Reentrant(group.spawner(), dropped.clone());
        assert!(!group.spawner().spawn(async move {
            let _capture = capture;
            std::future::pending::<()>().await;
        }));
        assert!(dropped.load(Ordering::Acquire));
    }
}
