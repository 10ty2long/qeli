use super::*;
use std::time::{Duration, Instant};
const LIMIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn already_stopped_never_polls_preparation_or_mutates() {
    let result = prepared(
        async {
            panic!("read-only preparation must not start");
            #[allow(unreachable_code)]
            Ok(|| Ok::<_, anyhow::Error>(()))
        },
        async {},
    )
    .await
    .unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn cancelling_read_only_preparation_drops_it_without_mutation() {
    struct Read(Arc<AtomicBool>);
    impl Drop for Read {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let resource = Read(dropped.clone());
    let (started, ready) = tokio::sync::oneshot::channel();
    let result = prepared(
        async move {
            let _resource = resource;
            started.send(()).unwrap();
            std::future::pending::<()>().await;
            Ok(|| -> anyhow::Result<()> { panic!("mutation must not start") })
        },
        async {
            ready.await.unwrap();
        },
    )
    .await
    .unwrap();
    assert_eq!(result, None);
    assert!(dropped.load(Ordering::Acquire));
}

#[tokio::test]
async fn admitted_work_remains_responsive_and_finishes_after_stop() {
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, blocked) = mpsc::channel();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let until = Instant::now();
    let mut task = tokio::spawn(prepared(
        async move {
            Ok(move || {
                started.send(()).unwrap();
                blocked.recv_timeout(LIMIT)?;
                Ok(42)
            })
        },
        async {
            let _ = stopped.await;
        },
    ));
    ready.await.unwrap();
    let _ = stop.send(());
    let still_owned = tokio::time::timeout(Duration::from_millis(40), &mut task)
        .await
        .is_err();
    let responsive = until.elapsed() < Duration::from_secs(1);
    let _ = release.send(());
    assert_eq!(task.await.unwrap().unwrap(), Some(42));
    assert!(still_owned && responsive);
}

#[tokio::test]
async fn a_late_operation_error_is_not_hidden_by_stop() {
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, blocked) = mpsc::channel();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(prepared(
        async move {
            Ok(move || -> anyhow::Result<()> {
                started.send(()).unwrap();
                blocked.recv_timeout(LIMIT)?;
                anyhow::bail!("firewall state remains unknown");
            })
        },
        async {
            let _ = stopped.await;
        },
    ));
    ready.await.unwrap();
    let _ = stop.send(());
    release.send(()).unwrap();
    assert_eq!(
        task.await.unwrap().unwrap_err().to_string(),
        "firewall state remains unknown"
    );
}

#[tokio::test]
async fn preparation_failure_never_invokes_a_mutation() {
    let result = prepared(
        async { Err::<fn() -> anyhow::Result<()>, _>(anyhow::anyhow!("resolution failed")) },
        std::future::pending::<()>(),
    )
    .await;
    assert_eq!(result.unwrap_err().to_string(), "resolution failed");
}

#[test]
fn abandoning_admitted_firewall_work_joins_without_removing_its_barrier() {
    let protected = Arc::new(AtomicBool::new(false));
    let policy = protected.clone();
    let (started, ready) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let owner = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut future = Box::pin(prepared(
            async move {
                Ok(move || {
                    started.send(()).unwrap();
                    blocked.recv_timeout(LIMIT)?;
                    policy.store(true, Ordering::Release);
                    Ok(())
                })
            },
            std::future::pending::<()>(),
        ));
        runtime.block_on(std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(future.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        }));
        drop(future);
        done.send(()).unwrap();
    });
    ready.recv_timeout(LIMIT).unwrap();
    let early = finished.recv_timeout(Duration::from_millis(50)).is_ok();
    release.send(()).unwrap();
    owner.join().unwrap();
    finished.recv_timeout(LIMIT).unwrap();
    assert!(!early);
    assert!(
        protected.load(Ordering::Acquire),
        "abandonment must not silently lift protection"
    );
}
