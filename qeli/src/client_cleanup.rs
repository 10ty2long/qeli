//! Release the host egress barrier only after core and forwarding cleanup succeed.
pub(crate) fn routing(
    core_stop: anyhow::Result<()>,
    kill_switch: bool,
    remove_forwarding: impl FnOnce() -> anyhow::Result<()>,
    release_kill_switch: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let core_stopped = core_stop.is_ok();
    // Even after a core fault, try to withdraw forwarding permits. Never release the
    // egress barrier unless both cleanup stages completed successfully.
    let forwarding = remove_forwarding();
    let forwarding_removed = forwarding.is_ok();
    if let Err(error) = with_cleanup_error(core_stop, forwarding) {
        let retained = if !kill_switch {
            ""
        } else if core_stopped {
            "; kill-switch retained because forwarding/NAT cleanup did not complete"
        } else if forwarding_removed {
            "; kill-switch retained because transport core teardown did not complete"
        } else {
            "; kill-switch retained because transport core teardown and forwarding/NAT cleanup did not complete"
        };
        let message = format!("host cleanup failed: {error}{retained}");
        return Err(error.context(message));
    }
    if kill_switch {
        release_kill_switch()
            .map_err(|error| anyhow::anyhow!("host firewall cleanup failed: {error}"))?;
    }
    Ok(())
}

/// Keep both causes visible in ordinary diagnostics, preserving the primary error type.
pub(crate) fn with_cleanup_error(
    result: anyhow::Result<()>,
    cleanup: anyhow::Result<()>,
) -> anyhow::Result<()> {
    match (result, cleanup) {
        (Ok(()), cleanup) => cleanup,
        (result, Ok(())) => result,
        (Err(error), Err(cleanup)) => {
            let message = format!("{error}; teardown also failed: {cleanup}");
            Err(error.context(message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[test]
    fn failed_forwarding_cleanup_keeps_the_egress_barrier() {
        let forwarding_remains = Cell::new(true);
        let barrier_installed = Cell::new(true);
        let error = routing(
            Ok(()),
            true,
            || {
                assert!(forwarding_remains.get());
                anyhow::bail!("IPv6 NAT delete failed")
            },
            || {
                barrier_installed.set(false);
                Ok(())
            },
        )
        .unwrap_err();
        assert!(forwarding_remains.get());
        assert!(
            barrier_installed.get(),
            "cleanup failure must not open host egress"
        );
        assert!(error.to_string().contains("IPv6 NAT delete failed"));
        assert!(error.to_string().contains("kill-switch retained"));
    }

    #[test]
    fn successful_cleanup_removes_forwarding_before_releasing_barrier() {
        let calls = RefCell::new(Vec::new());
        routing(
            Ok(()),
            true,
            || {
                calls.borrow_mut().push("forwarding");
                Ok(())
            },
            || {
                assert_eq!(*calls.borrow(), vec!["forwarding"]);
                calls.borrow_mut().push("kill-switch");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(*calls.borrow(), vec!["forwarding", "kill-switch"]);
    }

    #[test]
    fn barrier_release_failure_is_not_hidden_or_reported_as_retention() {
        let removed = Cell::new(false);
        let error = routing(
            Ok(()),
            true,
            || {
                removed.set(true);
                Ok(())
            },
            || {
                assert!(removed.get());
                anyhow::bail!("OUTPUT jump delete failed")
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("OUTPUT jump delete failed"));
        // Release may have partially succeeded: do not falsely promise protection.
        assert!(!error.to_string().contains("kill-switch retained"));
    }

    #[test]
    fn disabled_kill_switch_is_never_touched_or_claimed_as_retained() {
        for forwarding_ok in [true, false] {
            let result = routing(
                Ok(()),
                false,
                || {
                    if forwarding_ok {
                        Ok(())
                    } else {
                        anyhow::bail!("gateway cleanup failed")
                    }
                },
                || panic!("disabled kill-switch was touched"),
            );
            assert_eq!(result.is_ok(), forwarding_ok);
            if let Err(error) = result {
                assert!(!error.to_string().contains("kill-switch retained"));
            }
        }
    }

    fn busy_core() -> crate::transport_core::ClientCore {
        use crate::transport_core::{ClientCore, CoreOptions};
        let ini = format!(
            "[qeli]\nserver=127.0.0.1:443\nproto=tcp\nuser=test\npass=secret\nkey={}\nmode=fake-tls\n",
            "11".repeat(32)
        );
        let mut core = ClientCore::new(
            &ini,
            CoreOptions {
                event_capacity: 2,
                ..CoreOptions::default()
            },
        )
        .unwrap();
        // Created and Connecting fill a real bounded core queue without private test hooks.
        core.start().unwrap();
        core
    }

    #[test]
    fn core_queue_backpressure_retains_barrier_until_stop_really_completes() {
        use crate::transport_core::{ClientState, CoreError};
        let mut core = busy_core();
        let stop = core.stop().map_err(anyhow::Error::from);
        assert!(matches!(
            stop.as_ref().unwrap_err().downcast_ref(),
            Some(CoreError::EventQueueFull)
        ));
        assert_eq!(core.state(), ClientState::Connecting);
        let forwarded = Cell::new(true);
        let barrier = Cell::new(true);
        let error = routing(
            stop,
            true,
            || {
                forwarded.set(false);
                Ok(())
            },
            || {
                barrier.set(false);
                Ok(())
            },
        )
        .unwrap_err();
        assert!(
            !forwarded.get(),
            "core error must not skip withdrawal of forwarding permits"
        );
        assert!(
            barrier.get(),
            "an incomplete core stop must not release host egress"
        );
        assert!(matches!(
            error.downcast_ref(),
            Some(CoreError::EventQueueFull)
        ));
        assert!(error
            .to_string()
            .contains("transport core teardown did not complete"));

        // Model explicit recovery; the policy itself must not silently retry a generation.
        while core.poll_event().is_some() {}
        let stop = core.stop().map_err(anyhow::Error::from);
        assert_eq!(core.state(), ClientState::Stopped);
        routing(
            stop,
            true,
            || Ok(()),
            || {
                barrier.set(false);
                Ok(())
            },
        )
        .unwrap();
        assert!(!barrier.get());
    }

    #[test]
    fn core_and_forwarding_failures_are_both_visible_without_releasing_barrier() {
        use crate::transport_core::CoreError;
        let mut core = busy_core();
        let error = routing(
            core.stop().map_err(anyhow::Error::from),
            true,
            || anyhow::bail!("IPv6 NAT delete failed"),
            || panic!("incomplete cleanup must retain the kill-switch"),
        )
        .unwrap_err();
        let text = error.to_string();
        assert!(text.contains("event queue is full"));
        assert!(text.contains("IPv6 NAT delete failed"));
        assert!(text.contains("core teardown and forwarding/NAT cleanup did not complete"));
        assert!(matches!(
            error.downcast_ref(),
            Some(CoreError::EventQueueFull)
        ));
    }

    #[test]
    fn startup_failure_survives_successful_rollback() {
        use crate::transport_core::CoreError;
        let mut core = busy_core();
        let startup = core.start().map_err(anyhow::Error::from);
        assert!(startup.is_err());
        while core.poll_event().is_some() {}
        let barrier = Cell::new(true);
        let cleanup = routing(
            core.stop().map_err(anyhow::Error::from),
            true,
            || Ok(()),
            || {
                barrier.set(false);
                Ok(())
            },
        );
        let error = with_cleanup_error(startup, cleanup).unwrap_err();
        assert!(
            !barrier.get(),
            "a successfully stopped core can release the barrier"
        );
        assert!(matches!(
            error.downcast_ref(),
            Some(CoreError::InvalidState {
                operation: "start",
                ..
            })
        ));
    }

    #[test]
    fn startup_and_cleanup_failures_keep_both_causes() {
        use crate::transport_core::CoreError;
        let mut core = busy_core();
        let startup = core.start().map_err(anyhow::Error::from);
        let cleanup = routing(
            core.stop().map_err(anyhow::Error::from),
            true,
            || anyhow::bail!("NAT rollback failed"),
            || panic!("failed stop must retain kill-switch"),
        );
        let error = with_cleanup_error(startup, cleanup).unwrap_err();
        let text = error.to_string();
        assert!(text.contains("start"));
        assert!(text.contains("event queue is full"));
        assert!(text.contains("NAT rollback failed"));
        assert!(text.contains("kill-switch retained"));
        assert!(matches!(
            error.downcast_ref(),
            Some(CoreError::InvalidState { .. })
        ));
    }

    #[test]
    fn core_failure_does_not_claim_disabled_barrier_or_report_success() {
        let error = routing(
            Err(anyhow::anyhow!("core stop failed")),
            false,
            || Ok(()),
            || panic!("disabled barrier was touched"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("core stop failed"));
        assert!(!error.to_string().contains("kill-switch retained"));
    }

    #[test]
    fn cleanup_error_combination_preserves_success_and_each_single_error() {
        use crate::transport_core::CoreError;
        with_cleanup_error(Ok(()), Ok(())).unwrap();
        for primary_fails in [true, false] {
            let error = anyhow::Error::new(CoreError::EventQueueFull);
            let result = if primary_fails {
                with_cleanup_error(Err(error), Ok(()))
            } else {
                with_cleanup_error(Ok(()), Err(error))
            };
            assert!(matches!(
                result.unwrap_err().downcast_ref(),
                Some(CoreError::EventQueueFull)
            ));
        }
    }
}
