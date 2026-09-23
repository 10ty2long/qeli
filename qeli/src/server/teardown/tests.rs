use super::*;

#[test]
fn report_preserves_drop_failure_and_primary_failure_without_second_cleanup() {
    struct Guard(Report, Arc<std::sync::atomic::AtomicUsize>);
    impl Drop for Guard {
        fn drop(&mut self) {
            self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.0
                .record("TUN delete", Err(anyhow::anyhow!("permission denied")));
        }
    }
    let report = Report::default();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    drop(Guard(report.clone(), calls.clone()));
    report.record("generation", Err(anyhow::anyhow!("bind failed")));
    let message = report.result().unwrap_err().to_string();
    assert!(message.contains("permission denied") && message.contains("bind failed"));
    assert_eq!(report.result().unwrap_err().to_string(), message);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn poisoned_report_keeps_existing_evidence() {
    let report = Report::default();
    report.record("TUN", Err(anyhow::anyhow!("earlier failure")));
    let other = report.clone();
    let _ = std::panic::catch_unwind(move || {
        let _guard = other.0.lock().unwrap();
        panic!("fixture");
    });
    report.record("queues", Err(anyhow::anyhow!("later failure")));
    let message = report.result().unwrap_err().to_string();
    assert!(message.contains("earlier failure") && message.contains("later failure"));
}

#[test]
fn empty_threads_complete_without_waiting() {
    let mut wakes = 0;
    stop_threads(Vec::new(), Duration::ZERO, || wakes += 1).unwrap();
    assert_eq!(wakes, 1);
}

#[test]
fn normal_wake_joins_thread_and_releases_its_resource() {
    let resource = Arc::new(());
    let weak = Arc::downgrade(&resource);
    let (send, receive) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let _resource = resource;
        receive.recv().unwrap();
    });
    let mut send = Some(send);
    stop_threads(vec![handle], Duration::from_secs(2), || {
        if let Some(send) = send.take() {
            send.send(()).unwrap();
        }
    })
    .unwrap();
    assert!(weak.upgrade().is_none());
}

#[test]
fn timeout_does_not_block_join_and_still_reports_finished_thread_panic() {
    let panicked = std::thread::Builder::new()
        .name("q-reader".into())
        .spawn(|| panic!("queue fixture panic"))
        .unwrap();
    let ready = Instant::now() + Duration::from_secs(2);
    while !panicked.is_finished() {
        assert!(Instant::now() < ready);
        std::thread::sleep(Duration::from_millis(1));
    }
    let (send, receive) = std::sync::mpsc::channel();
    let (done, completed) = std::sync::mpsc::channel();
    let blocked = std::thread::spawn(move || {
        receive.recv().unwrap();
        done.send(()).unwrap();
    });
    let start = Instant::now();
    let error = stop_threads(vec![blocked, panicked], Duration::ZERO, || {})
        .unwrap_err()
        .to_string();
    send.send(()).unwrap();
    completed.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(
        error.contains("did not stop") && error.contains("queue fixture panic"),
        "{error}"
    );
}

#[test]
fn transient_setup_failure_can_restart_only_after_successful_cleanup() {
    let outcome = Outcome::new(Err(anyhow::anyhow!("port busy")), Ok(()));
    assert!(outcome.can_restart());
    assert!(outcome.result().is_err());
    assert!(outcome
        .into_result()
        .unwrap_err()
        .to_string()
        .contains("port busy"));
}

#[test]
fn old_cleanup_failure_reaches_worker_instead_of_entering_backoff() {
    let outcome = Outcome::new(Ok(()), Err(anyhow::anyhow!("TUN thread retains fd")));
    assert!(!outcome.can_restart());
    assert!(outcome
        .into_result()
        .unwrap_err()
        .to_string()
        .contains("retains fd"));
}

#[test]
fn primary_error_does_not_hide_unsafe_cleanup() {
    let outcome = Outcome::new(
        Err(anyhow::anyhow!("listener failed")),
        Err(anyhow::anyhow!("NAT remains")),
    );
    assert!(!outcome.can_restart());
    let error = outcome.into_result().unwrap_err().to_string();
    assert!(error.contains("listener failed") && error.contains("NAT remains"));
    let healthy = Outcome::new(Ok(()), Ok(()));
    assert!(healthy.can_restart());
    assert!(healthy.into_result().is_ok());
}
