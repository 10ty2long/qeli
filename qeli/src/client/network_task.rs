//! Joined network transactions on fresh threads, inheriting the caller's OS context.
//! The worker retains its result until the async caller explicitly adopts it. A lost
//! waiter rolls it back on that worker before releasing the join/outer namespace lease.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::JoinHandle;

struct Context {
    #[cfg(target_os = "linux")]
    namespaces: [std::fs::File; 2],
}
impl Context {
    fn capture() -> anyhow::Result<Self> {
        Ok(Self {
            #[cfg(target_os = "linux")]
            namespaces: [
                std::fs::File::open("/proc/thread-self/ns/net")?,
                std::fs::File::open("/proc/thread-self/ns/mnt")?,
            ],
        })
    }
    fn verify(&self) -> anyhow::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            for (file, name) in self.namespaces.iter().zip(["net", "mnt"]) {
                let expected = file.metadata()?;
                let actual = std::fs::metadata(format!("/proc/thread-self/ns/{name}"))?;
                anyhow::ensure!(
                    (expected.dev(), expected.ino()) == (actual.dev(), actual.ino()),
                    "network transaction {name} namespace changed"
                );
            }
        }
        Ok(())
    }
}

struct Job<T> {
    adopt: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<Option<anyhow::Result<T>>>>,
}
impl<T> Job<T> {
    fn decide(&mut self, accept: bool) {
        if let Some(adopt) = self.adopt.take() {
            if accept {
                let _ = adopt.send(());
            }
        }
    }
    fn finish(mut self) -> anyhow::Result<T> {
        self.thread
            .take()
            .expect("network transaction thread")
            .join()
            .map_err(|_| anyhow::anyhow!("network transaction worker panicked"))?
            .ok_or_else(cancelled)?
    }
}
impl<T> Drop for Job<T> {
    fn drop(&mut self) {
        // Closing admission asks the ORIGINAL worker to drop the unadopted result.
        // This synchronous fallback is deliberate: Drop cannot leave mutations running
        // after an outer TUN/namespace lease has been released. Normal stop uses the
        // shared cancellation flag and awaits completion without blocking the executor.
        self.adopt.take();
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("network transaction worker panicked during rollback");
            }
        }
    }
}
fn cancelled() -> anyhow::Error {
    std::io::Error::new(
        std::io::ErrorKind::Interrupted,
        "network transaction cancelled before adoption",
    )
    .into()
}

pub(crate) async fn run<T: Send + 'static>(
    cancel: Arc<AtomicBool>,
    work: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> anyhow::Result<T> {
    if cancel.load(Ordering::Acquire) {
        return Err(cancelled());
    }
    let context = Arc::new(Context::capture()?);
    let worker_context = context.clone();
    let worker_cancel = cancel.clone();
    let (ready, waiting) = tokio::sync::oneshot::channel();
    let (finished, completion) = tokio::sync::oneshot::channel();
    let (adopt, decision) = mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("qeli-network-plan".into())
        .spawn(move || {
            let result = worker_context.verify().and_then(|()| {
                if worker_cancel.load(Ordering::Acquire) {
                    Err(cancelled())
                } else {
                    work()
                }
            });
            let _ = ready.send(());
            let output = if decision.recv().is_ok() {
                Some(result)
            } else {
                drop(result); // TunGuard/partial ownership is released in its original context.
                None
            };
            let _ = finished.send(()); // Includes rejected-result rollback before notifying.
            output
        })?;
    let mut job = Job {
        adopt: Some(adopt),
        thread: Some(thread),
    };
    // A panic drops ready; finish still joins and reports the worker failure.
    let _ = waiting.await;
    let valid = context.verify();
    if valid.is_ok() && !cancel.load(Ordering::Acquire) {
        // No await after acceptance: ownership transfers in this poll. Work has already
        // finished; the worker can only return the value, never perform late rollback.
        job.decide(true);
        job.finish()
    } else {
        job.decide(false);
        let _ = completion.await; // Rejected values may need slow network rollback.
        let result = job.finish();
        valid?;
        result
    }
}

#[cfg(test)]
#[path = "network_task/tests.rs"]
mod tests;
