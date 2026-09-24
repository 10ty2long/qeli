//! One command/admission deadline for a gateway setup, refresh or cleanup attempt.
use std::{
    io,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};
#[derive(Clone, Copy)]
pub(super) struct Budget {
    pub(super) until: Instant,
}
impl Budget {
    pub(super) fn new() -> Self {
        #[cfg(test)]
        if let Some(until) = DEADLINE.with(|slot| slot.get()) {
            return Self { until };
        }
        Self {
            until: Instant::now() + Duration::from_secs(15),
        }
    }
    fn remaining(self) -> io::Result<Duration> {
        self.until
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "router operation deadline expired; unresolved ownership retained for retry",
                )
            })
    }
    pub(super) fn check(self) -> io::Result<()> {
        self.remaining().map(|_| ())
    }
    pub(super) fn lock<T>(self, lock: &Mutex<T>) -> io::Result<MutexGuard<'_, T>> {
        loop {
            self.check()?;
            let acquired = match lock.try_lock() {
                Ok(guard) => Some(guard),
                Err(std::sync::TryLockError::Poisoned(error)) => Some(error.into_inner()),
                Err(std::sync::TryLockError::WouldBlock) => None,
            };
            if let Some(guard) = acquired {
                self.check()?;
                return Ok(guard);
            }
            std::thread::sleep(self.remaining()?.min(Duration::from_millis(10)));
        }
    }
}
#[cfg(test)]
thread_local! { static DEADLINE: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) }; }
#[cfg(test)]
pub(super) fn with_deadline<T>(until: Instant, run: impl FnOnce() -> T) -> T {
    struct Reset(Option<Instant>);
    impl Drop for Reset {
        fn drop(&mut self) {
            DEADLINE.with(|slot| slot.set(self.0));
        }
    }
    let _reset = Reset(DEADLINE.with(|slot| slot.replace(Some(until))));
    run()
}
