//! Worker process supervision, independent of privileged Linux profile setup.
use std::{future::Future, io, pin::Pin, time::Duration};
use tokio::{
    process::Child,
    sync::{mpsc, watch},
};

/// Command the web panel (in the supervisor) sends to the supervisor loop to
/// act on the data-plane worker child process.
#[derive(Debug, Clone, Copy)]
pub enum WorkerCmd {
    /// Restart the worker (SIGTERM + respawn) — applies profile/config changes.
    Restart,
    /// SIGHUP the worker to hot-reload users / brute-force thresholds.
    ReloadUsers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerSignal {
    Stop,
    ReloadUsers,
}

pub(crate) struct SupervisorPolicy {
    pub(crate) initial_backoff: Duration,
    pub(crate) maximum_backoff: Duration,
    pub(crate) stable_after: Duration,
    pub(crate) shutdown_grace: Duration,
}
impl Default for SupervisorPolicy {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(1),
            maximum_backoff: Duration::from_secs(30),
            stable_after: Duration::from_secs(30),
            shutdown_grace: Duration::from_secs(60),
        }
    }
}

/// A worker handle never leaves the supervision future. Its PID is published only while
/// owned; cancellation kills the child rather than detaching an independently waiting task.
struct OwnedWorker<'a, Report: FnMut(Option<u32>)> {
    child: Child,
    report: &'a mut Report,
}
impl<Report: FnMut(Option<u32>)> Drop for OwnedWorker<'_, Report> {
    fn drop(&mut self) {
        // No-op after a successful wait. On cancellation, request kill and hand reaping
        // to Tokio when Child drops; synchronous Drop cannot itself await process exit.
        let _ = self.child.start_kill();
        (self.report)(None);
    }
}

/// Shared watch semantics for profile stop and retry waits: a true value OR loss of the
/// owner stops the generation. A false update alone never requests shutdown.
pub(crate) async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}

async fn wait_for_retry(
    delay: Duration,
    commands: &mut mpsc::Receiver<WorkerCmd>,
    mut stop: Pin<&mut impl Future<Output = ()>>,
) -> bool {
    let timer = tokio::time::sleep(delay);
    tokio::pin!(timer);
    loop {
        if commands.is_closed() {
            return false;
        }
        tokio::select! {
            // A pending stop beats an expired retry timer. The timer beats a stream of
            // reload commands, so busy panel clients cannot prevent the next spawn.
            biased;
            _ = &mut stop => return false,
            _ = &mut timer => return true,
            command = commands.recv() => match command {
                Some(WorkerCmd::Restart) => return true,
                Some(WorkerCmd::ReloadUsers) => {}, // next worker reads the latest file
                None => return false,
            }
        }
    }
}

async fn wait_for_deadline(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

fn request_termination(
    child: &mut Child,
    signal: &mut impl FnMut(&mut Child, WorkerSignal) -> io::Result<()>,
    deadline: &mut Option<tokio::time::Instant>,
    grace: Duration,
) {
    if deadline.is_none() {
        *deadline = Some(tokio::time::Instant::now() + grace);
        if let Err(error) = signal(child, WorkerSignal::Stop) {
            log::warn!("supervisor: worker termination signal failed: {error}");
        }
    }
}

pub(crate) async fn supervise(
    mut spawn: impl FnMut() -> io::Result<Child>,
    mut signal: impl FnMut(&mut Child, WorkerSignal) -> io::Result<()>,
    mut report_pid: impl FnMut(Option<u32>),
    commands: &mut mpsc::Receiver<WorkerCmd>,
    stop: impl Future<Output = ()>,
    policy: SupervisorPolicy,
) -> io::Result<()> {
    tokio::pin!(stop);
    let mut backoff = policy.initial_backoff;
    let mut delay = Duration::ZERO;
    loop {
        if !wait_for_retry(delay, commands, stop.as_mut()).await {
            return Ok(());
        }
        // Commands queued while no worker existed all refer to files the new worker will
        // read. Do not signal a freshly spawned generation for those old requests. Bound
        // draining to the queue capacity so concurrent senders cannot starve spawning.
        for _ in 0..commands.max_capacity() {
            match commands.try_recv() {
                Ok(_) => {}
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => return Ok(()),
            }
        }
        if commands.is_closed() {
            return Ok(());
        }
        let child = match spawn() {
            Ok(child) => child,
            Err(error) => {
                log::error!("supervisor: failed to spawn worker: {error} — retry in {backoff:?}");
                delay = backoff;
                backoff = backoff.saturating_mul(2).min(policy.maximum_backoff);
                continue;
            }
        };
        report_pid(child.id());
        log::info!(
            "supervisor: data-plane worker started (pid {})",
            child.id().unwrap_or(0)
        );
        let mut worker = OwnedWorker {
            child,
            report: &mut report_pid,
        };
        let started = tokio::time::Instant::now();
        let mut stopping = false;
        let mut restarting = false;
        let mut deadline = None;
        loop {
            tokio::select! {
                biased;
                _ = &mut stop, if !stopping => {
                    stopping = true;
                    log::info!("supervisor: stopping worker");
                    request_termination(&mut worker.child, &mut signal, &mut deadline, policy.shutdown_grace);
                },
                status = worker.child.wait() => {
                    match status {
                        Ok(status) => log::info!("supervisor: worker exited ({status})"),
                        Err(error) => return Err(error),
                    }
                    break;
                },
                _ = wait_for_deadline(deadline) => {
                    log::warn!("supervisor: worker did not stop within {:?} — killing and reaping it", policy.shutdown_grace);
                    worker.child.kill().await?;
                    break;
                },
                command = commands.recv(), if !stopping => match command {
                    Some(WorkerCmd::ReloadUsers) if deadline.is_none() => {
                        if let Err(error) = signal(&mut worker.child, WorkerSignal::ReloadUsers) {
                            log::warn!("supervisor: worker reload signal failed: {error}");
                        }
                    },
                    Some(WorkerCmd::ReloadUsers) => {}, // exiting worker; next reads latest users
                    command => {
                        stopping = command.is_none();
                        restarting = !stopping;
                        // Repeated Restart never extends the grace period or sends a HUP
                        // into teardown. Closing the command channel disables recv above.
                        if deadline.is_none() {
                            log::info!("supervisor: requesting worker stop (restart={restarting})");
                        }
                        request_termination(&mut worker.child, &mut signal, &mut deadline, policy.shutdown_grace);
                    },
                },
            }
        }
        // Drop clears metrics BEFORE any retry delay. All waits/signals use this same
        // Child, so a detached waiter can no longer reap and recycle the stored PID.
        drop(worker);
        if stopping {
            return Ok(());
        }
        if restarting {
            backoff = policy.initial_backoff;
            delay = Duration::ZERO;
        } else {
            if started.elapsed() >= policy.stable_after {
                backoff = policy.initial_backoff;
            }
            delay = backoff;
            backoff = backoff.saturating_mul(2).min(policy.maximum_backoff);
            log::warn!("supervisor: worker stopped unexpectedly — respawning in {delay:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Stdio,
        sync::{
            atomic::{AtomicU32, AtomicUsize, Ordering},
            Arc, Mutex,
        },
    };
    use tokio::{
        process::{ChildStdin, Command},
        sync::{oneshot, watch},
        time::timeout,
    };
    const DEADLINE: Duration = Duration::from_secs(5);

    #[test]
    fn worker_fixture() {
        if std::env::var_os("QELI_SUPERVISOR_TEST_CHILD").is_some() {
            // A real isolated child: no network or host changes; exits when the owner
            // closes stdin. This also cleans up the baseline's detached waiter on failure.
            let _ = std::io::copy(&mut std::io::stdin(), &mut std::io::sink());
        }
    }

    fn spawn_fixture(held: &Arc<Mutex<Option<ChildStdin>>>) -> io::Result<Child> {
        let mut child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "server_supervisor::tests::worker_fixture",
                "--nocapture",
            ])
            .env("QELI_SUPERVISOR_TEST_CHILD", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        *held.lock().unwrap() = child.stdin.take();
        Ok(child)
    }

    async fn until(mut condition: impl FnMut() -> bool) {
        timeout(DEADLINE, async {
            while !condition() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("supervisor did not reach expected state");
    }

    fn policy() -> SupervisorPolicy {
        SupervisorPolicy {
            initial_backoff: Duration::from_secs(30),
            maximum_backoff: Duration::from_secs(30),
            shutdown_grace: Duration::from_millis(30),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn stop_interrupts_spawn_failure_retry() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let counter = attempts.clone();
        let (_commands, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let worker = tokio::spawn(async move {
            supervise(
                || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(io::ErrorKind::PermissionDenied.into())
                },
                |_, _| Ok(()),
                |_| {},
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| attempts.load(Ordering::SeqCst) == 1).await;
        stop_tx.send(()).unwrap();
        let mut worker = worker;
        let stopped = timeout(Duration::from_millis(300), &mut worker).await;
        worker.abort();
        assert!(stopped.is_ok(), "spawn failure retry ignored stop");
        stopped.unwrap().unwrap().unwrap();
    }

    #[tokio::test]
    async fn worker_exit_clears_pid_before_backoff() {
        let pid = Arc::new(AtomicU32::new(0));
        let published = pid.clone();
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let (_commands, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let worker = tokio::spawn(async move {
            supervise(
                || spawn_fixture(&child_stdin),
                |_, _| Ok(()),
                |value| published.store(value.unwrap_or(0), Ordering::SeqCst),
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| pid.load(Ordering::SeqCst) != 0).await;
        held.lock().unwrap().take(); // child exits normally
        let cleared = timeout(
            Duration::from_secs(1),
            until(|| pid.load(Ordering::SeqCst) == 0),
        )
        .await;
        let _ = stop_tx.send(());
        timeout(DEADLINE, worker).await.unwrap().unwrap().unwrap();
        assert!(
            cleared.is_ok(),
            "dead worker PID remains published during retry backoff"
        );
    }

    #[tokio::test]
    async fn cancelling_supervisor_clears_owned_child() {
        let pid = Arc::new(AtomicU32::new(0));
        let published = pid.clone();
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let (_commands, mut rx) = mpsc::channel(8);
        let worker = tokio::spawn(async move {
            supervise(
                || spawn_fixture(&child_stdin),
                |_, _| Ok(()),
                |value| published.store(value.unwrap_or(0), Ordering::SeqCst),
                &mut rx,
                std::future::pending(),
                policy(),
            )
            .await
        });
        until(|| pid.load(Ordering::SeqCst) != 0).await;
        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        assert_eq!(
            pid.load(Ordering::SeqCst),
            0,
            "cancelled supervisor still publishes an unowned child"
        );
        let mut stdin = held.lock().unwrap().take().unwrap();
        timeout(DEADLINE, async {
            use tokio::io::AsyncWriteExt;
            while stdin.write_all(&[0]).await.is_ok() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("cancelled supervisor left its child alive");
    }

    #[tokio::test]
    async fn unresponsive_worker_stop_is_bounded() {
        let pid = Arc::new(AtomicU32::new(0));
        let published = pid.clone();
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let (_commands, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let mut worker = tokio::spawn(async move {
            supervise(
                || spawn_fixture(&child_stdin),
                |_, _| Ok(()),
                |value| published.store(value.unwrap_or(0), Ordering::SeqCst),
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| pid.load(Ordering::SeqCst) != 0).await;
        stop_tx.send(()).unwrap();
        let stopped = timeout(Duration::from_secs(1), &mut worker).await;
        worker.abort();
        assert!(
            stopped.is_ok(),
            "worker ignoring termination blocks shutdown indefinitely"
        );
        stopped.unwrap().unwrap().unwrap();
        assert_eq!(pid.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn already_requested_stop_prevents_spawn() {
        let (_tx, mut rx) = mpsc::channel(8);
        supervise(
            || panic!("spawn after stop"),
            |_, _| Ok(()),
            |_| {},
            &mut rx,
            async {},
            policy(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn closed_command_channel_prevents_spawn() {
        let (tx, mut rx) = mpsc::channel(8);
        drop(tx);
        supervise(
            || panic!("spawn without owner"),
            |_, _| Ok(()),
            |_| {},
            &mut rx,
            std::future::pending(),
            policy(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn closed_command_channel_stops_live_worker_once() {
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let pid = Arc::new(AtomicU32::new(0));
        let published = pid.clone();
        let stops = Arc::new(AtomicUsize::new(0));
        let stopped = stops.clone();
        let (tx, mut rx) = mpsc::channel(8);
        let worker = tokio::spawn(async move {
            supervise(
                || spawn_fixture(&child_stdin),
                |child, signal| {
                    assert_eq!(signal, WorkerSignal::Stop);
                    stopped.fetch_add(1, Ordering::SeqCst);
                    child.start_kill()
                },
                |value| published.store(value.unwrap_or(0), Ordering::SeqCst),
                &mut rx,
                std::future::pending(),
                policy(),
            )
            .await
        });
        until(|| pid.load(Ordering::SeqCst) != 0).await;
        drop(tx);
        timeout(DEADLINE, worker).await.unwrap().unwrap().unwrap();
        assert_eq!(stops.load(Ordering::SeqCst), 1);
        assert_eq!(pid.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn reload_keeps_generation_and_restart_coalesces_queued_commands() {
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let attempts = Arc::new(AtomicUsize::new(0));
        let spawned = attempts.clone();
        let reloads = Arc::new(AtomicUsize::new(0));
        let reloaded = reloads.clone();
        let stops = Arc::new(AtomicUsize::new(0));
        let stopped = stops.clone();
        let (tx, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let worker = tokio::spawn(async move {
            supervise(
                || {
                    let child = spawn_fixture(&child_stdin)?;
                    spawned.fetch_add(1, Ordering::SeqCst);
                    Ok(child)
                },
                |child, signal| match signal {
                    WorkerSignal::Stop => {
                        stopped.fetch_add(1, Ordering::SeqCst);
                        child.start_kill()
                    }
                    WorkerSignal::ReloadUsers => {
                        reloaded.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }
                },
                |_| {},
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| attempts.load(Ordering::SeqCst) == 1).await;
        tx.send(WorkerCmd::ReloadUsers).await.unwrap();
        until(|| reloads.load(Ordering::SeqCst) == 1).await;
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        // Queue all three without yielding: the next generation already reads those files.
        tx.try_send(WorkerCmd::Restart).unwrap();
        tx.try_send(WorkerCmd::Restart).unwrap();
        tx.try_send(WorkerCmd::ReloadUsers).unwrap();
        until(|| attempts.load(Ordering::SeqCst) == 2).await;
        stop_tx.send(()).unwrap();
        timeout(DEADLINE, worker).await.unwrap().unwrap().unwrap();
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(reloads.load(Ordering::SeqCst), 1);
        assert_eq!(stops.load(Ordering::SeqCst), 2); // restart and final stop
    }

    #[tokio::test]
    async fn apply_interrupts_spawn_backoff_without_reloading_fresh_worker() {
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let attempts = Arc::new(AtomicUsize::new(0));
        let spawned = attempts.clone();
        let reloads = Arc::new(AtomicUsize::new(0));
        let reloaded = reloads.clone();
        let (tx, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let worker = tokio::spawn(async move {
            supervise(
                || {
                    if spawned.fetch_add(1, Ordering::SeqCst) == 0 {
                        return Err(io::ErrorKind::NotFound.into());
                    }
                    spawn_fixture(&child_stdin)
                },
                |child, signal| match signal {
                    WorkerSignal::Stop => child.start_kill(),
                    WorkerSignal::ReloadUsers => {
                        reloaded.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }
                },
                |_| {},
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| attempts.load(Ordering::SeqCst) == 1).await;
        tx.try_send(WorkerCmd::ReloadUsers).unwrap();
        tx.try_send(WorkerCmd::Restart).unwrap();
        tx.try_send(WorkerCmd::ReloadUsers).unwrap();
        // The policy has a 30-second retry, so reaching the next attempt within DEADLINE
        // establishes that Apply wakes the backoff instead of waiting for its timer.
        until(|| attempts.load(Ordering::SeqCst) == 2).await;
        stop_tx.send(()).unwrap();
        timeout(DEADLINE, worker).await.unwrap().unwrap().unwrap();
        assert_eq!(reloads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn repeated_restart_does_not_extend_termination_deadline() {
        let held = Arc::new(Mutex::new(None));
        let child_stdin = held.clone();
        let attempts = Arc::new(AtomicUsize::new(0));
        let spawned = attempts.clone();
        let stops = Arc::new(AtomicUsize::new(0));
        let stopped = stops.clone();
        let (tx, mut rx) = mpsc::channel(8);
        let (stop_tx, stop_rx) = oneshot::channel();
        let worker = tokio::spawn(async move {
            supervise(
                || {
                    let child = spawn_fixture(&child_stdin)?;
                    spawned.fetch_add(1, Ordering::SeqCst);
                    Ok(child)
                },
                |_, signal| {
                    assert_eq!(signal, WorkerSignal::Stop);
                    stopped.fetch_add(1, Ordering::SeqCst);
                    Ok(()) // deliberately ignore the graceful termination request
                },
                |_| {},
                &mut rx,
                async {
                    let _ = stop_rx.await;
                },
                policy(),
            )
            .await
        });
        until(|| attempts.load(Ordering::SeqCst) == 1).await;
        for _ in 0..8 {
            tx.try_send(WorkerCmd::Restart).unwrap();
        }
        until(|| attempts.load(Ordering::SeqCst) == 2).await;
        stop_tx.send(()).unwrap();
        timeout(DEADLINE, worker).await.unwrap().unwrap().unwrap();
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(stops.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn watch_stop_accepts_preexisting_true_or_closed_owner() {
        let (_tx, mut rx) = watch::channel(true);
        timeout(DEADLINE, wait_for_shutdown(&mut rx)).await.unwrap();
        let (tx, mut rx) = watch::channel(false);
        drop(tx);
        timeout(DEADLINE, wait_for_shutdown(&mut rx)).await.unwrap();
    }

    #[tokio::test]
    async fn watch_false_updates_and_cancelled_wait_do_not_lose_stop() {
        let (tx, mut rx) = watch::channel(false);
        tx.send(false).unwrap();
        let mut wait = Box::pin(wait_for_shutdown(&mut rx));
        std::future::poll_fn(|cx| {
            assert!(wait.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(wait);
        tx.send(true).unwrap();
        timeout(DEADLINE, wait_for_shutdown(&mut rx)).await.unwrap();
    }
    #[tokio::test]
    async fn closed_owner_with_queued_restart_prevents_retry() {
        let (tx, mut rx) = mpsc::channel(2);
        tx.try_send(WorkerCmd::Restart).unwrap();
        tx.try_send(WorkerCmd::ReloadUsers).unwrap();
        drop(tx);
        let mut stop = Box::pin(std::future::pending::<()>());
        assert!(
            !wait_for_retry(Duration::from_secs(30), &mut rx, stop.as_mut()).await,
            "queued Restart must not outlive the owner of the command channel"
        );
    }
}
