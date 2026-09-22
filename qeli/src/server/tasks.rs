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
}
