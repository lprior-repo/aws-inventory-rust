//! Behavior: terminal_errors_are_not_retried.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn terminal_errors_are_not_retried() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    for error in [
        ApiError::Denied("read permission absent".into()),
        ApiError::Permanent("bad request".into()),
    ] {
        let expected = error.clone();
        let result: Result<(), ApiError> = executor
            .execute("terminal", || async { Err(error.clone()) })
            .await;
        assert_eq!(result, Err(expected));
    }
    let stats = executor.stats();
    assert_eq!(stats.attempts, 2);
    assert_eq!(stats.retries, 0);
    assert_eq!(stats.failures, 2);
    Ok(())
}
