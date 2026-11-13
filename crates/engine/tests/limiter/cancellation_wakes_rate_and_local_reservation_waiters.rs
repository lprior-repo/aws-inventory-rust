//! Behavior: cancellation_wakes_rate_and_local_reservation_waiters.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn cancellation_wakes_rate_and_local_reservation_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 1,
            ..config()
        },
        42,
    )?;
    executor.execute("paced", || async { Ok(()) }).await?;
    let rate_waiter = executor.execute("paced", || async { Ok(()) });
    let local_waiter = executor.execute("paced", || async { Ok(()) });
    let cancel = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (first, second, ()) = tokio::join!(rate_waiter, local_waiter, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(second, Err(ApiError::Cancelled));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().cancelled, 2);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
