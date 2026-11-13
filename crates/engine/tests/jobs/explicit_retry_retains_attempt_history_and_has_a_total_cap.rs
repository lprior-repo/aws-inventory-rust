//! Behavior: explicit_retry_retains_attempt_history_and_has_a_total_cap.
use super::*;

#[tokio::test]
pub(super) async fn explicit_retry_retains_attempt_history_and_has_a_total_cap() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("retry", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store
        .complete(
            &lease,
            Completion {
                state: TerminalJobState::Partial,
                result: Some(json!({"status":"partial"})),
                error: Some("denied scope".into()),
            },
            101,
        )
        .await?;
    assert!(store.claim("w", "t2", 102, 10000).await?.is_none());
    let queued = store.retry("retry", 2, 102).await?;
    assert_eq!(queued.max_attempts, 3);
    assert_eq!(queued.attempt, 1);
    let next = store
        .claim("w", "t2", 102, 10000)
        .await?
        .ok_or("retry claim")?;
    assert_eq!(next.job.attempt, 2);
    store
        .complete(
            &next,
            Completion {
                state: TerminalJobState::Failed,
                result: None,
                error: Some("failed".into()),
            },
            103,
        )
        .await?;
    assert!(matches!(
        store.retry("retry", 31, 104).await,
        Err(JobError::Limit(_))
    ));
    assert_eq!(store.history("retry").await?.len(), 2);
    store.close().await;
    Ok(())
}
