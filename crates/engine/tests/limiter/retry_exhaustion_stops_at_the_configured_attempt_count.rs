use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn retry_exhaustion_stops_at_the_configured_attempt_count() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let result: Result<(), ApiError> = executor
        .execute("failing", || async {
            Err(ApiError::Transient("retry me".into()))
        })
        .await;
    assert!(matches!(result, Err(ApiError::Exhausted(message)) if message.contains("retry me")));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 3);
    assert_eq!(stats.retries, 2);
    assert_eq!(stats.failures, 1);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}
