use super::*;

#[tokio::test]
pub(super) async fn errors_are_utf8_bounded_and_terminal_failures_are_explicit() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("error", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    let done = store
        .complete(
            &lease,
            Completion {
                state: TerminalJobState::Failed,
                result: None,
                error: Some("🦀".repeat(9000)),
            },
            101,
        )
        .await?;
    assert_eq!(done.last_error.as_ref().map(String::len), Some(8192));
    assert_eq!(done.state, JobState::Failed);
    store.close().await;
    Ok(())
}
