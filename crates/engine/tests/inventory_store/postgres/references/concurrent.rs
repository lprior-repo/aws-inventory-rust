use super::{TestResult, claim, collect, edges, relationships, run, support, work};
use inventory_engine::inventory::{DatabaseUrl, PageCommit, PgStore, PgWriteConfig};
use sqlx::PgPool;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_concurrent_reviewed_writers_when_target_changes_then_no_reference_is_lost_stale_or_deadlocked()
-> TestResult {
    run(concurrent).await
}

async fn concurrent(store: &PgStore, pool: &PgPool, url: &DatabaseUrl) -> TestResult {
    let parents = ["parent-one", "parent-two"]
        .into_iter()
        .map(|id| support::observation(id, id))
        .collect::<TestResult<Vec<_>>>()?;
    collect(store, "parents", parents, Vec::new()).await?;
    store
        .seed_works(&[
            work("concurrent", Some(support::resource("parent-one")?))?,
            work("concurrent", Some(support::resource("parent-two")?))?,
        ])
        .await?;
    let other = PgStore::open(url, PgWriteConfig::default()).await?;
    let result = changes(store, &other, pool).await;
    other.close().await;
    result
}

async fn changes(store: &PgStore, other: &PgStore, pool: &PgPool) -> TestResult {
    pair(store, other, 1, "target-name").await?;
    assert_eq!(edges(pool).await?, ["target"]);
    pair(store, other, 2, "renamed").await?;
    assert!(edges(pool).await?.is_empty());
    pair(store, other, 3, "target-name").await?;
    assert_eq!(edges(pool).await?, ["target"]);
    Ok(())
}

async fn pair(store: &PgStore, other: &PgStore, token: u8, value: &str) -> TestResult {
    let mut source = page(store, "parent-one", "source", "source-name", token).await?;
    relationships(&mut source, false)?;
    let target = page(other, "parent-two", "target", value, token).await?;
    let (left, right) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::join!(store.commit_page(source), other.commit_page(target))
    })
    .await?;
    left?;
    right?;
    Ok(())
}

async fn page(
    store: &PgStore,
    parent: &str,
    id: &str,
    value: &str,
    token: u8,
) -> TestResult<PageCommit> {
    let lease = claim(store, "concurrent", 100).await?;
    assert_eq!(lease.scope().parent(), Some(&support::resource(parent)?));
    Ok(support::page(
        lease,
        vec![support::observation(id, value)?],
        false,
        Some(vec![token]),
        101,
    ))
}
