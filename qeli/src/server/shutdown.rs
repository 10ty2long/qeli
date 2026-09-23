//! Final worker stop outcome, shared with host tests without Linux process exit.

pub(crate) fn result(
    fatal_reason: Option<String>,
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
        assert!(result(None, Ok(()), Ok(())).is_ok());
    }

    #[test]
    fn network_failure_is_not_masked_by_successful_usage_flush() {
        let outcome = result(None, Err(anyhow::anyhow!("DNS rule remains")), Ok(()));
        assert_eq!(i32::from(outcome.is_err()), 1);
        assert!(outcome
            .unwrap_err()
            .to_string()
            .contains("DNS rule remains"));
    }

    #[test]
    fn usage_failure_still_fails_an_otherwise_clean_stop() {
        let outcome = result(None, Ok(()), Err(anyhow::anyhow!("disk full")));
        assert_eq!(i32::from(outcome.is_err()), 1);
        assert!(outcome.unwrap_err().to_string().contains("disk full"));
    }

    #[test]
    fn worker_failure_survives_successful_cleanup_and_flush() {
        let outcome = result(Some("control server stopped".into()), Ok(()), Ok(()));
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
            Err(anyhow::anyhow!(detail.clone())),
            Err(anyhow::anyhow!(detail)),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(message.chars().filter(|c| *c == 'я').count(), 3 * 2048);
        assert!(message.len() < 13000);
    }
}
