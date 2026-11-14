use super::*;

#[tokio::test]
pub(super) async fn competing_database_connections_claim_only_once() -> TestResult {
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("queue.sqlite");
    let a = JobStore::open(&path).await?;
    let b = JobStore::open(&path).await?;
    a.submit(spec("single", 3), 100).await?;
    let (one, two) = tokio::join!(
        a.claim("a", "token-a", 100, 10000),
        b.claim("b", "token-b", 100, 10000)
    );
    let claimed = [one?, two?].into_iter().flatten().count();
    assert_eq!(claimed, 1);
    assert_eq!(a.history("single").await?.len(), 1);
    a.close().await;
    b.close().await;
    Ok(())
}
