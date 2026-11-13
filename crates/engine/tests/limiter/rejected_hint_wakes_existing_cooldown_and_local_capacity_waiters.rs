//! Behavior: rejected_hint_wakes_existing_cooldown_and_local_capacity_waiters.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn rejected_hint_wakes_existing_cooldown_and_local_capacity_waiters() -> TestResult
{
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 3,
            per_bucket_concurrency: 3,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let beginning = Instant::now();
    let first = executor.execute("shared", || async {
        sleep(Duration::from_millis(100)).await;
        Err::<(), _>(directed(
            RetryKind::Transient,
            RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
        ))
    });
    let second = executor.execute("shared", || async {
        sleep(Duration::from_millis(200)).await;
        Err::<(), _>(directed(RetryKind::Throttled, RetryAfter::Invalid))
    });
    let peer = async {
        sleep(Duration::from_millis(150)).await;
        executor.execute("shared", || async { Ok(()) }).await
    };
    let local = async {
        sleep(Duration::from_millis(160)).await;
        executor.execute("shared", || async { Ok(()) }).await
    };
    let outcomes = timeout(Duration::from_millis(300), async {
        tokio::join!(first, second, peer, local)
    })
    .await?;
    for outcome in [outcomes.0, outcomes.1, outcomes.2, outcomes.3] {
        assert!(matches!(outcome, Err(ApiError::RetryAfterRejected(_))));
    }
    assert!(Instant::now().duration_since(beginning) < Duration::from_millis(300));
    assert_eq!(executor.stats().attempts, 2);
    assert_eq!(executor.stats().in_flight, 0);
    assert_eq!(executor.stats().server_delays, 1);
    assert_eq!(executor.stats().rejected_retry_hints, 1);
    Ok(())
}
