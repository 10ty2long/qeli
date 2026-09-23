//! Final worker stop outcome, shared with host tests without Linux process exit.

/// Bounded failure evidence retained even when a shutdown waiter is cancelled.
#[derive(Clone, Default)]
pub(crate) struct Failures {
    messages: Vec<String>,
    omitted: usize,
}

impl Failures {
    pub(crate) fn record(&mut self, label: &str, result: anyhow::Result<()>) {
        if let Err(error) = result {
            if self.messages.len() < 8 {
                let label: String = label.chars().take(256).collect();
                let detail: String = error.to_string().chars().take(2048).collect();
                self.messages.push(format!("{label}: {detail}"));
            } else {
                self.omitted = self.omitted.saturating_add(1);
            }
        }
    }

    /// These sets are deliberately aborted during profile shutdown. Cancellation is
    /// expected; a panic or a completed service error must still be preserved.
    pub(crate) fn record_aborted_task(
        &mut self,
        label: &str,
        result: Result<anyhow::Result<()>, tokio::task::JoinError>,
    ) {
        match result {
            Ok(result) => self.record(label, result),
            Err(error) if error.is_cancelled() => {}
            Err(error) => self.record(label, Err(error.into())),
        }
    }

    pub(crate) fn result(&self) -> anyhow::Result<()> {
        if self.messages.is_empty() {
            return Ok(());
        }
        let mut message = self.messages.join("; ");
        if self.omitted != 0 {
            message.push_str(&format!("; {} additional errors", self.omitted));
        }
        anyhow::bail!("cleanup failed: {message}")
    }
}

/// Profile supervisors are asked to stop normally, never aborted in this path.
/// Drain every one even after an error, panic, or unexpected cancellation.
pub(crate) async fn drain_profiles(
    profiles: &mut tokio::task::JoinSet<anyhow::Result<()>>,
    failures: &mut Failures,
) {
    while let Some(joined) = profiles.join_next().await {
        failures.record(
            "profile supervisor",
            match joined {
                Ok(result) => result,
                Err(error) => Err(error.into()),
            },
        );
    }
}

pub(crate) fn result(
    fatal_reason: Option<String>,
    task_cleanup: anyhow::Result<()>,
    owned_cleanup: anyhow::Result<()>,
    usage_flush: anyhow::Result<()>,
) -> anyhow::Result<()> {
    let mut failures = Vec::new();
    let mut record = |label: &str, reason: String| {
        let detail: String = reason.chars().take(2048).collect();
        failures.push(format!("{label}: {detail}"));
    };
    if let Some(reason) = fatal_reason {
        record("worker", reason);
    }
    if let Err(error) = task_cleanup {
        record("profile/worker task cleanup", error.to_string());
    }
    if let Err(error) = owned_cleanup {
        record("owned network cleanup", error.to_string());
    }
    if let Err(error) = usage_flush {
        record("usage shutdown flush", error.to_string());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("Server shutdown failed: {}", failures.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_final_retry_allows_a_clean_stop() {
        assert!(result(None, Ok(()), Ok(()), Ok(())).is_ok());
    }

    #[test]
    fn network_failure_is_not_masked_by_successful_usage_flush() {
        let outcome = result(
            None,
            Ok(()),
            Err(anyhow::anyhow!("DNS rule remains")),
            Ok(()),
        );
        assert_eq!(i32::from(outcome.is_err()), 1);
        assert!(outcome
            .unwrap_err()
            .to_string()
            .contains("DNS rule remains"));
    }

    #[test]
    fn usage_failure_still_fails_an_otherwise_clean_stop() {
        let outcome = result(None, Ok(()), Ok(()), Err(anyhow::anyhow!("disk full")));
        assert_eq!(i32::from(outcome.is_err()), 1);
        assert!(outcome.unwrap_err().to_string().contains("disk full"));
    }

    #[test]
    fn worker_failure_survives_successful_cleanup_and_flush() {
        let outcome = result(
            Some("control server stopped".into()),
            Ok(()),
            Ok(()),
            Ok(()),
        );
        assert_eq!(i32::from(outcome.is_err()), 1);
        assert!(outcome
            .unwrap_err()
            .to_string()
            .contains("control server stopped"));
    }

    #[test]
    fn concurrent_causes_remain_visible_in_the_final_error() {
        let message = result(
            Some("supervisor failed".into()),
            Ok(()),
            Err(anyhow::anyhow!("sysctl restore denied")),
            Err(anyhow::anyhow!("disk full")),
        )
        .unwrap_err()
        .to_string();
        for cause in ["supervisor failed", "sysctl restore denied", "disk full"] {
            assert!(message.contains(cause), "{message}");
        }
    }

    #[test]
    fn large_unicode_diagnostics_are_bounded_per_cause() {
        let detail = "я".repeat(10000);
        let message = result(
            Some(detail.clone()),
            Ok(()),
            Err(anyhow::anyhow!(detail.clone())),
            Err(anyhow::anyhow!(detail)),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(message.chars().filter(|c| *c == 'я').count(), 3 * 2048);
        assert!(message.len() < 13000);
    }

    #[test]
    fn task_failure_is_part_of_worker_exit_even_when_network_and_flush_succeed() {
        let result = result(None, Err(anyhow::anyhow!("queue panic")), Ok(()), Ok(()));
        assert_eq!(i32::from(result.is_err()), 1);
        assert!(result.unwrap_err().to_string().contains("queue panic"));
    }

    #[test]
    fn every_shutdown_category_remains_visible() {
        let error = result(
            Some("primary fixture".into()),
            Err(anyhow::anyhow!("tasks fixture")),
            Err(anyhow::anyhow!("network fixture")),
            Err(anyhow::anyhow!("flush fixture")),
        )
        .unwrap_err()
        .to_string();
        for cause in [
            "primary fixture",
            "tasks fixture",
            "network fixture",
            "flush fixture",
        ] {
            assert!(error.contains(cause), "{error}");
        }
    }

    #[test]
    fn failure_evidence_is_bounded_and_success_does_not_erase_it() {
        let mut failures = Failures::default();
        for _ in 0..20 {
            failures.record(&"я".repeat(1000), Err(anyhow::anyhow!("я".repeat(10000))));
        }
        failures.record("success", Ok(()));
        let error = failures.result().unwrap_err().to_string();
        assert!(error.contains("12 additional errors"));
        assert_eq!(
            error.chars().filter(|c| *c == 'я').count(),
            8 * (256 + 2048)
        );
        assert_eq!(failures.result().unwrap_err().to_string(), error);
    }

    #[tokio::test]
    async fn profile_drain_keeps_failures_across_cancellation_and_drains_every_profile() {
        let mut profiles = tokio::task::JoinSet::new();
        profiles.spawn(async {
            anyhow::bail!("profile result fixture");
        });
        profiles.spawn(async {
            panic!("profile panic fixture");
        });
        let cancelled = profiles.spawn(std::future::pending());
        cancelled.abort();
        let (send, receive) = tokio::sync::oneshot::channel();
        profiles.spawn(async {
            receive.await.unwrap();
            Ok(())
        });
        tokio::task::yield_now().await;
        let mut failures = Failures::default();
        let mut first = Box::pin(drain_profiles(&mut profiles, &mut failures));
        use std::future::Future;
        std::future::poll_fn(|cx| {
            assert!(first.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(first);
        assert_eq!(profiles.len(), 1);
        send.send(()).unwrap();
        drain_profiles(&mut profiles, &mut failures).await;
        let error = failures.result().unwrap_err().to_string();
        assert!(
            error.contains("profile result fixture")
                && error.contains("profile panic fixture")
                && error.contains("cancelled"),
            "{error}"
        );
        assert!(profiles.is_empty());
    }
}
