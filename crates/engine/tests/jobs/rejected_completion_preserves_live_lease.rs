use super::*;

#[tokio::test]
pub(super) async fn rejected_completion_preserves_live_lease() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("rollback", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    assert!(TerminalJobState::try_from(JobState::Queued).is_err());
    assert!(TerminalJobState::try_from(JobState::Running).is_err());
    assert!(
        store
            .complete(
                &lease,
                Completion {
                    state: TerminalJobState::Succeeded,
                    result: Some(json!({"too_big":"x".repeat(1048576)})),
                    error: None
                },
                101
            )
            .await
            .is_err()
    );
    assert_eq!(store.get("rollback").await?.state, JobState::Running);
    assert_eq!(
        store
            .history("rollback")
            .await?
            .first()
            .map(|v| v.state.as_str()),
        Some("running")
    );
    store.complete(&lease, success(), 102).await?;
    store.close().await;
    Ok(())
}
