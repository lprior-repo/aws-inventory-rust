//! Behavior: cancellation_wakes_active_calls_and_global_waiters.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn cancellation_wakes_active_calls_and_global_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let active = executor.execute("active", pending::<Result<(), ApiError>>);
    let queued = executor.execute("queued", pending::<Result<(), ApiError>>);
    let cancel = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (first, second, ()) = tokio::join!(active, queued, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(second, Err(ApiError::Cancelled));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 1);
    assert_eq!(stats.cancelled, 2);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}
