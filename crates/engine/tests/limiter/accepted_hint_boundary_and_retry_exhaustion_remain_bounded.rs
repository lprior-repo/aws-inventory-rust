//! Behavior: accepted_hint_boundary_and_retry_exhaustion_remain_bounded.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn accepted_hint_boundary_and_retry_exhaustion_remain_bounded() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let starts = Mutex::new(Vec::new());
    let result = executor
        .execute("bounded", || async {
            record(&starts)?;
            Err::<(), _>(directed(
                RetryKind::Timeout,
                RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
            ))
        })
        .await;
    assert!(
        matches!(result, Err(ApiError::Exhausted(message)) if message.contains("timeout") && message.contains("fixture server delay"))
    );
    assert_eq!(recorded(&starts)?.len(), 3);
    spaced(
        &recorded(&starts)?,
        Duration::from_millis(MAX_RETRY_AFTER_MS),
    );
    assert_eq!(executor.stats().attempts, 3);
    assert_eq!(executor.stats().retries, 2);
    assert_eq!(executor.stats().timeouts, 3);
    assert_eq!(executor.stats().server_delays, 3);
    Ok(())
}
