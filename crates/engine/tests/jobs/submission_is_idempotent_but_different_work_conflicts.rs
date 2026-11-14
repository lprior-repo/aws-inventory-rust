use super::*;

#[tokio::test]
pub(super) async fn submission_is_idempotent_but_different_work_conflicts() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    let first = store.submit(spec("job-one", 3), 100).await?;
    let mut repeated = spec("job-one", 3);
    repeated.available_at = 900;
    let again = store.submit(repeated, 200).await?;
    assert_eq!(again.created_at, first.created_at);
    assert_eq!(again.available_at, 100);
    let mut other = spec("job-one", 3);
    other.payload = json!({"profile":"different"});
    assert!(matches!(
        store.submit(other, 300).await,
        Err(JobError::Conflict)
    ));
    assert!(matches!(
        store.submit(spec("job-one", 4), 300).await,
        Err(JobError::Conflict)
    ));
    assert_eq!(store.list(10).await?.len(), 1);
    store.close().await;
    Ok(())
}
