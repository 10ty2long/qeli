//! Release the host egress barrier only after forwarding cleanup has succeeded.
pub(crate) fn routing(
    kill_switch: bool,
    remove_forwarding: impl FnOnce() -> anyhow::Result<()>,
    release_kill_switch: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    if let Err(error) = remove_forwarding() {
        if kill_switch {
            anyhow::bail!(
                "host firewall cleanup failed: {error}; kill-switch retained because forwarding/NAT cleanup did not complete"
            );
        }
        anyhow::bail!("host firewall cleanup failed: {error}");
    }
    if kill_switch {
        release_kill_switch()
            .map_err(|error| anyhow::anyhow!("host firewall cleanup failed: {error}"))?;
    }
    Ok(())
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
}
