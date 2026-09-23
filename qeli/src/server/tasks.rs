//! Ownership and shutdown of one profile generation.

use std::sync::Arc;
use tokio::task::JoinSet;

/// All nested session/query tasks belonging to one profile generation.
/// Admission closes before cancellation; shutdown waits for task destructors, even when
/// an earlier waiter was cancelled or another owner is already waiting for shutdown.
#[derive(Clone)]
pub(crate) struct ProfileTasks {
    profile: Arc<str>,
    inner: Arc<std::sync::Mutex<ProfileTasksInner>>,
    // JoinSet remembers one join waker. Serialize waiters without holding a blocking mutex
    // across .await, and keep every pending handle in `inner` when a waiter is cancelled.
    shutdown_lock: Arc<tokio::sync::Mutex<()>>,
}

struct ProfileTasksInner {
    stopping: bool,
    tasks: JoinSet<()>,
}

impl ProfileTasks {
    pub(crate) fn new(profile: &str) -> Self {
        Self {
            profile: Arc::from(profile),
            inner: Arc::new(std::sync::Mutex::new(ProfileTasksInner {
                stopping: false,
                tasks: JoinSet::new(),
            })),
            shutdown_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    fn report_result(&self, result: Result<(), tokio::task::JoinError>) {
        if let Err(error) = result {
            if !error.is_cancelled() {
                log::error!("Profile '{}': child task failed: {}", self.profile, error);
            }
        }
    }

    pub(crate) fn spawn<F>(&self, future: F) -> bool
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if inner.stopping {
            return false;
        }
        // Reap notified completions rather than scanning every live session for every DNS
        // query. Keep panic diagnostics instead of silently dropping finished JoinHandles.
        while let Some(result) = inner.tasks.try_join_next() {
            self.report_result(result);
        }
        inner.tasks.spawn(future);
        true
    }

    pub(crate) fn abort_all(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.stopping = true;
        inner.tasks.abort_all();
    }

    pub(crate) async fn shutdown(&self) {
        self.abort_all();
        let _waiter = self.shutdown_lock.lock().await;
        while let Some(result) = std::future::poll_fn(|cx| {
            self.inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .tasks
                .poll_join_next(cx)
        })
        .await
        {
            self.report_result(result);
        }
    }
}

/// Critical services and ingress listeners remain owned by the outer profile wrapper.
/// Keeping these sets outside startup ensures every `?` joins them before TUN/firewall
/// teardown, just like a normal stop. Dropping a JoinSet only requests cancellation.
#[derive(Default)]
pub(crate) struct ProfileServices {
    pub(crate) services: JoinSet<anyhow::Result<()>>,
    pub(crate) listeners: JoinSet<anyhow::Result<()>>,
}

impl ProfileServices {
    pub(crate) async fn shutdown(&mut self, tasks: &ProfileTasks) {
        tasks.abort_all();
        self.listeners.abort_all();
        self.services.abort_all();
        while self.listeners.join_next().await.is_some() {}
        while self.services.join_next().await.is_some() {}
        tasks.shutdown().await;
    }
}

/// Worker periodic services finish their current cycle before profile resources disappear.
/// Required services are monitored: silently losing quota enforcement is a worker failure.
/// Optional observers (e.g. an unarmed packet trace) may finish normally.
pub(crate) struct WorkerServices {
    tasks: JoinSet<()>,
    names: std::collections::HashMap<tokio::task::Id, (&'static str, bool)>,
    shutdown: tokio::sync::watch::Sender<bool>,
}

impl WorkerServices {
    pub(crate) fn new() -> Self {
        Self {
            tasks: JoinSet::new(),
            names: Default::default(),
            shutdown: tokio::sync::watch::channel(false).0,
        }
    }

    pub(crate) fn subscribe(&self) -> tokio::sync::watch::Receiver<bool> {
        self.shutdown.subscribe()
    }

    pub(crate) fn spawn<F>(&mut self, name: &'static str, required: bool, future: F) -> bool
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        if *self.shutdown.borrow() {
            return false;
        }
        let handle = self.tasks.spawn(future);
        self.names.insert(handle.id(), (name, required));
        true
    }

    pub(crate) async fn next_failure(&mut self) -> String {
        loop {
            let Some(result) = self.tasks.join_next_with_id().await else {
                return std::future::pending().await;
            };
            let id = match &result {
                Ok((id, ())) => *id,
                Err(error) => error.id(),
            };
            let (name, required) = self.names.remove(&id).expect("owned worker service");
            let failed = result.is_err();
            let reason = match result {
                Ok(_) => format!("worker service '{name}' stopped unexpectedly"),
                Err(error) => format!("worker service '{name}' failed: {error}"),
            };
            if required {
                return reason;
            }
            if failed {
                log::warn!("{reason}");
            }
        }
    }

    pub(crate) fn request_shutdown(&self) {
        self.shutdown.send_replace(true);
    }

    pub(crate) async fn shutdown(&mut self) {
        self.request_shutdown();
        // JoinSet retains pending handles if this waiter is cancelled. Never abort a
        // normal quota sweep halfway through session removal / route / lease cleanup.
        while let Some(result) = self.tasks.join_next_with_id().await {
            let id = match &result {
                Ok((id, ())) => *id,
                Err(error) => error.id(),
            };
            let (name, _) = self.names.remove(&id).expect("owned worker service");
            if let Err(error) = result {
                log::error!("worker service '{name}' failed during shutdown: {error}");
            }
        }
    }
}

impl Drop for WorkerServices {
    fn drop(&mut self) {
        self.request_shutdown();
        // JoinSet drop is only an emergency abort fallback for outer cancellation.
        // Normal worker teardown must call shutdown() to finish ongoing transactions.
    }
}

pub(crate) async fn worker_tick(
    tick: &mut tokio::time::Interval,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
) -> bool {
    tokio::select! {
        biased;
        _ = crate::server_supervisor::wait_for_shutdown(shutdown) => false,
        _ = tick.tick() => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    #[tokio::test]
    async fn profile_tasks_abort_join_and_close_admission() {
        struct DropSignal(Option<tokio::sync::oneshot::Sender<()>>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                if let Some(sender) = self.0.take() {
                    let _ = sender.send(());
                }
            }
        }

        let tasks = ProfileTasks::new("test");
        let (dropped_tx, dropped_rx) = tokio::sync::oneshot::channel();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        assert!(tasks.spawn(async move {
            let _signal = DropSignal(Some(dropped_tx));
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
        }));
        assert!(
            started_rx.await.is_ok(),
            "child task must start before shutdown"
        );

        tasks.shutdown().await;

        assert!(
            dropped_rx.await.is_ok(),
            "aborted child future must be dropped"
        );
        assert!(
            !tasks.spawn(async {}),
            "a closed generation must reject late child tasks"
        );
    }

    // Poll without yielding to Tokio: aborted children have not run their destructors yet.
    async fn poll_pending<F: Future>(future: std::pin::Pin<&mut F>) {
        let mut future = future;
        std::future::poll_fn(|cx| {
            assert!(
                future.as_mut().poll(cx).is_pending(),
                "shutdown returned before its children were dropped"
            );
            std::task::Poll::Ready(())
        })
        .await;
    }

    #[tokio::test]
    async fn concurrent_shutdown_waits_for_same_children() {
        let tasks = ProfileTasks::new("concurrent");
        tasks.spawn(std::future::pending());
        let mut first = Box::pin(tasks.shutdown());
        poll_pending(first.as_mut()).await;
        let mut second = Box::pin(tasks.shutdown());
        poll_pending(second.as_mut()).await;
        tokio::join!(first, second);
    }

    #[tokio::test]
    async fn cancelled_shutdown_keeps_children_joinable() {
        let tasks = ProfileTasks::new("cancelled");
        tasks.spawn(std::future::pending());
        let mut first = Box::pin(tasks.shutdown());
        poll_pending(first.as_mut()).await;
        drop(first);
        let mut retry = Box::pin(tasks.shutdown());
        poll_pending(retry.as_mut()).await;
        retry.await;
        assert!(!tasks.spawn(async {}));
    }
    #[tokio::test]
    async fn shutdown_joins_services_and_children_before_cleanup() {
        let tasks = ProfileTasks::new("setup-failure");
        let mut services = ProfileServices::default();
        let child_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let service_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listener_socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let child_addr = child_socket.local_addr().unwrap();
        let service_addr = service_socket.local_addr().unwrap();
        let listener_addr = listener_socket.local_addr().unwrap();
        tasks.spawn(async move {
            let _socket = child_socket;
            std::future::pending::<()>().await;
        });
        services.services.spawn(async move {
            let _socket = service_socket;
            std::future::pending().await
        });
        services.listeners.spawn(async move {
            let _socket = listener_socket;
            std::future::pending().await
        });
        // Same cleanup boundary as the outer run_profile wrapper after any startup `?`.
        services.shutdown(&tasks).await;
        let _child = tokio::net::UdpSocket::bind(child_addr).await.unwrap();
        let _service = tokio::net::UdpSocket::bind(service_addr).await.unwrap();
        let _listener = tokio::net::TcpListener::bind(listener_addr).await.unwrap();
        assert!(!tasks.spawn(async {}));
    }

    #[tokio::test]
    async fn cancelled_service_shutdown_can_be_resumed() {
        let tasks = ProfileTasks::new("cancelled-services");
        let mut services = ProfileServices::default();
        services.services.spawn(std::future::pending());
        services.listeners.spawn(std::future::pending());
        tasks.spawn(std::future::pending());
        let mut shutdown = Box::pin(services.shutdown(&tasks));
        poll_pending(shutdown.as_mut()).await;
        drop(shutdown);
        services.shutdown(&tasks).await;
        assert!(services.services.is_empty());
        assert!(services.listeners.is_empty());
        assert!(tasks.inner.lock().unwrap().tasks.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_spawn_and_shutdown_release_every_future() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Owned(Arc<AtomicUsize>);
        impl Drop for Owned {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::SeqCst);
            }
        }
        let live = Arc::new(AtomicUsize::new(0));
        let tasks = ProfileTasks::new("spawn-race");
        let start = Arc::new(tokio::sync::Barrier::new(9));
        let mut workers = JoinSet::new();
        for _ in 0..8 {
            let tasks = tasks.clone();
            let live = live.clone();
            let start = start.clone();
            workers.spawn(async move {
                start.wait().await;
                for _ in 0..128 {
                    live.fetch_add(1, Ordering::SeqCst);
                    let owned = Owned(live.clone());
                    tasks.spawn(async move {
                        let _owned = owned;
                        std::future::pending::<()>().await;
                    });
                    tokio::task::yield_now().await;
                }
            });
        }
        start.wait().await;
        tokio::task::yield_now().await;
        tasks.shutdown().await;
        while let Some(result) = workers.join_next().await {
            result.unwrap();
        }
        tasks.shutdown().await;
        assert_eq!(live.load(Ordering::SeqCst), 0);
        assert!(!tasks.spawn(async {}));
    }

    #[tokio::test]
    async fn finished_and_panicked_children_are_reaped() {
        let tasks = ProfileTasks::new("reap");
        tasks.spawn(async {});
        tasks.spawn(async {
            panic!("intentional child panic");
        });
        tokio::task::yield_now().await;
        tasks.spawn(std::future::pending());
        assert_eq!(tasks.inner.lock().unwrap().tasks.len(), 1);
        tasks.shutdown().await;
    }

    #[tokio::test]
    async fn worker_required_completion_is_failure() {
        let mut services = WorkerServices::new();
        services.spawn("quota", true, async {});
        let reason =
            tokio::time::timeout(std::time::Duration::from_secs(2), services.next_failure())
                .await
                .unwrap();
        assert!(reason.contains("quota") && reason.contains("stopped unexpectedly"));
        services.shutdown().await;
    }

    #[tokio::test]
    async fn worker_required_panic_is_failure() {
        let mut services = WorkerServices::new();
        services.spawn("quota", true, async { panic!("fixture failure") });
        let reason =
            tokio::time::timeout(std::time::Duration::from_secs(2), services.next_failure())
                .await
                .unwrap();
        assert!(reason.contains("quota") && reason.contains("fixture failure"));
        services.shutdown().await;
    }

    #[tokio::test]
    async fn worker_optional_completion_does_not_stop_required_service() {
        let mut services = WorkerServices::new();
        services.spawn("unarmed trace", false, async {});
        let mut shutdown = services.subscribe();
        services.spawn("quota", true, async move {
            crate::server_supervisor::wait_for_shutdown(&mut shutdown).await;
        });
        tokio::task::yield_now().await;
        let mut failure = Box::pin(services.next_failure());
        poll_pending(failure.as_mut()).await;
        drop(failure);
        services.shutdown().await;
        assert!(services.tasks.is_empty() && services.names.is_empty());
    }

    #[tokio::test]
    async fn worker_shutdown_finishes_cycle_before_releasing_resources() {
        let mut services = WorkerServices::new();
        let mut shutdown = services.subscribe();
        let cycles = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let shared_cycles = cycles.clone();
        let resource = Arc::new(());
        let weak = Arc::downgrade(&resource);
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = tokio::sync::oneshot::channel();
        services.spawn("quota transaction", true, async move {
            let _resource = resource;
            let mut tick = tokio::time::interval(std::time::Duration::from_millis(1));
            assert!(worker_tick(&mut tick, &mut shutdown).await);
            started_tx.send(()).unwrap();
            finish_rx.await.unwrap();
            shared_cycles.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            assert!(
                !worker_tick(&mut tick, &mut shutdown).await,
                "no new cycle after stop"
            );
        });
        started_rx.await.unwrap();
        let mut first = Box::pin(services.shutdown());
        poll_pending(first.as_mut()).await;
        assert!(
            weak.upgrade().is_some(),
            "in-flight transaction retains its resources"
        );
        drop(first); // Cancelling a shutdown waiter must keep the service joinable.
        finish_tx.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), services.shutdown())
            .await
            .unwrap();
        assert_eq!(cycles.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(weak.upgrade().is_none());
        services.shutdown().await; // idempotent
        assert!(!services.spawn("late admission", true, async {}));
    }

    #[tokio::test]
    async fn worker_tick_stop_wins_over_ready_tick_and_closed_owner() {
        let (tx, mut rx) = tokio::sync::watch::channel(true);
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(600));
        assert!(!worker_tick(&mut tick, &mut rx).await);
        tx.send_replace(false);
        assert!(worker_tick(&mut tick, &mut rx).await);
        drop(tx);
        assert!(!worker_tick(&mut tick, &mut rx).await);
    }

    #[tokio::test]
    async fn dropping_worker_owner_aborts_remaining_services() {
        struct DropSignal(Option<tokio::sync::oneshot::Sender<()>>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                let _ = self.0.take().unwrap().send(());
            }
        }
        let mut services = WorkerServices::new();
        let (dropped_tx, dropped_rx) = tokio::sync::oneshot::channel();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        services.spawn("fallback", true, async move {
            let _signal = DropSignal(Some(dropped_tx));
            started_tx.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        started_rx.await.unwrap();
        drop(services);
        tokio::time::timeout(std::time::Duration::from_secs(2), dropped_rx)
            .await
            .unwrap()
            .unwrap();
    }
}
