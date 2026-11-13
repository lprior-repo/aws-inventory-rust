//! Behavior: queue_and_inventory_database_paths_are_not_interchangeable.
use super::*;

#[tokio::test]
pub(super) async fn queue_and_inventory_database_paths_are_not_interchangeable() -> TestResult {
    let folder = tempfile::tempdir()?;
    let inventory = folder.path().join("inventory.sqlite");
    let queue = folder.path().join("jobs.sqlite");
    Store::open(&inventory).await?.close().await;
    assert!(matches!(
        JobStore::open(&inventory).await,
        Err(JobError::ForeignDatabase)
    ));
    JobStore::open(&queue).await?.close().await;
    assert!(Store::open(&queue).await.is_err());
    let reopened = Store::open(&inventory).await?;
    reopened.close().await;
    let reopened = JobStore::open(&queue).await?;
    reopened.close().await;
    Ok(())
}
