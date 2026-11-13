//! Behavior: rejected_hint_cancels_already_active_same_bucket_operations.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn rejected_hint_cancels_already_active_same_bucket_operations() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let first = executor.execute("shared", || async {
        sleep(Duration::from_millis(200)).await;
        Err::<(), _>(directed(RetryKind::Transient, RetryAfter::Invalid))
    });
    let peer = executor.execute("shared", pending::<Result<(), ApiError>>);
    let (first, peer) = timeout(Duration::from_millis(300), async {
        tokio::join!(first, peer)
    })
    .await?;
    assert!(matches!(first, Err(ApiError::RetryAfterRejected(_))));
    assert!(matches!(peer, Err(ApiError::RetryAfterRejected(_))));
    assert_eq!(executor.stats().attempts, 2);
    assert_eq!(executor.stats().timeouts, 0);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
