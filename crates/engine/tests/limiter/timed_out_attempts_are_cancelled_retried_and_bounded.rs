use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn timed_out_attempts_are_cancelled_retried_and_bounded() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            max_attempts: 2,
            attempt_timeout_ms: 10,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let result = executor
        .execute("timeout", pending::<Result<(), ApiError>>)
        .await;
    assert!(matches!(result, Err(ApiError::Exhausted(message)) if message.contains("timeout")));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 2);
    assert_eq!(stats.timeouts, 2);
    assert_eq!(stats.retries, 1);
    assert_eq!(stats.failures, 1);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}
