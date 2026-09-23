//! Finish client-owned background work before publishing the terminal status.
use tokio::task::JoinSet;

pub(crate) async fn finish(tasks: &mut JoinSet<()>, publish: impl FnOnce()) {
    tasks.abort_all();
    // Aborting does not wait for a synchronous publish already in progress. Keep the
    // handles in the owner's JoinSet until every writer ends, including after a waiter
    // is cancelled and retried. No older diagnostic snapshot can then overwrite final.
    while let Some(result) = tasks.join_next().await {
        if result.is_err_and(|error| !error.is_cancelled()) {
            log::error!("client background task failed during shutdown");
        }
    }
    publish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use std::time::Duration;
    const DEADLINE: Duration = Duration::from_secs(5);

    async fn blocked_writer(
        tasks: &mut JoinSet<()>,
        events: Arc<Mutex<Vec<&'static str>>>,
    ) -> std::sync::mpsc::Sender<()> {
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        tasks.spawn(async move {
            started.send(()).unwrap();
            // Simulate an in-flight synchronous status-file write. Aborting a Tokio task
            // cannot interrupt it; dropping release also unblocks on a failed assertion.
            let _ = blocked.recv();
            events.lock().unwrap().push("old snapshot");
        });
        tokio::time::timeout(DEADLINE, ready)
            .await
            .unwrap()
            .unwrap();
        release
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn final_publication_waits_for_an_inflight_writer() {
        let mut tasks = JoinSet::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let release = blocked_writer(&mut tasks, events.clone()).await;
        let output = events.clone();
        let mut finishing = tokio::spawn(async move {
            finish(&mut tasks, || output.lock().unwrap().push("final")).await;
            assert!(tasks.is_empty());
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(30), &mut finishing)
                .await
                .is_err()
        );
        assert!(events.lock().unwrap().is_empty());
        release.send(()).unwrap();
        tokio::time::timeout(DEADLINE, finishing)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(*events.lock().unwrap(), vec!["old snapshot", "final"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_finish_can_be_retried_without_losing_the_writer() {
        let mut tasks = JoinSet::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let release = blocked_writer(&mut tasks, events.clone()).await;
        assert!(tokio::time::timeout(
            Duration::from_millis(30),
            finish(&mut tasks, || panic!("published before writer ended"))
        )
        .await
        .is_err());
        assert!(!tasks.is_empty());
        release.send(()).unwrap();
        tokio::time::timeout(
            DEADLINE,
            finish(&mut tasks, || events.lock().unwrap().push("final")),
        )
        .await
        .unwrap();
        assert_eq!(*events.lock().unwrap(), vec!["old snapshot", "final"]);
    }

    #[tokio::test]
    async fn panic_does_not_skip_other_destructors_or_final_status() {
        struct Lease(Arc<AtomicBool>);
        impl Drop for Lease {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let released = Arc::new(AtomicBool::new(false));
        let lease = Lease(released.clone());
        let mut tasks = JoinSet::new();
        let (started, ready) = tokio::sync::oneshot::channel();
        tasks.spawn(async move {
            started.send(()).unwrap();
            panic!("fixture task panic");
        });
        tasks.spawn(async move {
            let _lease = lease;
            std::future::pending::<()>().await;
        });
        ready.await.unwrap();
        finish(&mut tasks, || assert!(released.load(Ordering::Acquire))).await;
        assert!(tasks.is_empty());
    }
}
