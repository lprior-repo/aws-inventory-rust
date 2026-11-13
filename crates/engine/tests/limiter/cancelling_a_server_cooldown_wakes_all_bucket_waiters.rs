//! Behavior: cancelling_a_server_cooldown_wakes_all_bucket_waiters.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn cancelling_a_server_cooldown_wakes_all_bucket_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let beginning = Instant::now();
    let first = executor.execute("shared", || async {
        Err::<(), _>(directed(
            RetryKind::Transient,
            RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
        ))
    });
    let peer = executor.execute("shared", || async { Ok(()) });
    let cancel = async {
        sleep(Duration::from_millis(50)).await;
        executor.cancel();
    };
    let (first, peer, ()) = tokio::join!(first, peer, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(peer, Err(ApiError::Cancelled));
    assert!(Instant::now().duration_since(beginning) < Duration::from_millis(100));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().in_flight, 0);
    assert_eq!(executor.stats().cancelled, 2);
    Ok(())
}
