use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn invalid_and_excessive_hints_close_only_the_affected_bucket() -> TestResult {
    for hint in [
        RetryAfter::Invalid,
        RetryAfter::Excessive,
        RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS.saturating_add(1))),
        RetryAfter::Delay(Duration::MAX),
    ] {
        let executor = ApiExecutor::with_seed(config(), 42)?;
        let calls = AtomicU32::new(0);
        let first = executor
            .execute("closed", || async {
                calls.fetch_add(1, Ordering::Relaxed);
                Err::<(), _>(directed(RetryKind::Throttled, hint))
            })
            .await;
        assert!(
            matches!(first, Err(ApiError::RetryAfterRejected(message)) if message.contains("fixture server delay"))
        );
        let peer = executor
            .execute("closed", || async {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await;
        assert!(matches!(peer, Err(ApiError::RetryAfterRejected(_))));
        executor.execute("other", || async { Ok(()) }).await?;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(executor.stats().attempts, 2);
        assert_eq!(executor.stats().throttles, 1);
        assert_eq!(executor.stats().rejected_retry_hints, 1);
        assert_eq!(executor.stats().server_delays, 0);
    }
    Ok(())
}
