//! Behavior: restart_preserves_claim_and_recovers_only_after_expiration.
use super::*;

#[tokio::test]
pub(super) async fn restart_preserves_claim_and_recovers_only_after_expiration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("restart.sqlite");
    let first = JobStore::open(&path).await?;
    first.submit(spec("persist", 2), 100).await?;
    let lease = first
        .claim("before", "token", 100, 10000)
        .await?
        .ok_or("first claim")?;
    first.close().await;
    let reopened = JobStore::open(&path).await?;
    assert_eq!(reopened.get("persist").await?.state, JobState::Running);
    assert!(
        reopened
            .claim("after", "new", 10099, 10000)
            .await?
            .is_none()
    );
    let next = reopened
        .claim("after", "new", 10100, 10000)
        .await?
        .ok_or("reclaim")?;
    assert_eq!(next.job.attempt, 2);
    assert!(matches!(
        reopened.complete(&lease, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    reopened.complete(&next, success(), 10101).await?;
    reopened.close().await;
    Ok(())
}
