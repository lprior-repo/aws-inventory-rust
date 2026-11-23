use super::{TestResult, pages};
use inventory_core::inventory::ScopeOutcome;
use inventory_core::record::{Exclusions, Value};
use inventory_engine::inventory::{ChangeStats, DatabaseUrl, PageCommit, PgStore, PgWriteConfig};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_two_facet_owners_when_pages_commit_concurrently_then_neither_field_is_lost()
-> TestResult {
    pages::run(concurrent).await
}

async fn concurrent(store: &PgStore, pool: &PgPool, url: &DatabaseUrl) -> TestResult {
    parents(store).await?;
    let (left, right) = work_pages(store).await?;
    let other = PgStore::open(url, PgWriteConfig::default()).await?;
    let (left, right) = tokio::join!(store.commit_page(left), other.commit_page(right));
    other.close().await;
    check_stats(left?.stats, right?.stats)?;
    check(pool).await
}

async fn work_pages(store: &PgStore) -> TestResult<(PageCommit, PageCommit)> {
    store
        .seed_works(&[
            pages::work("facets", Some(pages::key("parent_a")?))?,
            pages::work("facets", Some(pages::key("parent_b")?))?,
        ])
        .await?;
    let first = pages::claim(store, "facets", 5).await?;
    let second = pages::claim(store, "facets", 5).await?;
    let left = pages::observation("shared", [(1, Value::Bool(true))], Exclusions::none())?;
    let right = pages::observation("shared", [(2, Value::Int(7))], Exclusions::none())?;
    Ok((
        pages::page(first, vec![left], 1, 6),
        pages::page(second, vec![right], 2, 6),
    ))
}

fn check_stats(left: ChangeStats, right: ChangeStats) -> TestResult {
    let inserted = left
        .resources_inserted
        .checked_add(right.resources_inserted)
        .ok_or("count overflow")?;
    let changed = left
        .resources_changed
        .checked_add(right.resources_changed)
        .ok_or("count overflow")?;
    assert_eq!(inserted, 1);
    assert_eq!(changed, 1);
    Ok(())
}

async fn parents(store: &PgStore) -> TestResult {
    let lease = pages::seed(store, "parents").await?;
    let observations = ["parent_a", "parent_b"]
        .into_iter()
        .map(|id| pages::observation(id, [], Exclusions::none()))
        .collect::<TestResult<Vec<_>>>()?;
    store
        .commit_page(pages::page(lease, observations, 1, 2))
        .await?;
    let lease = pages::claim(store, "parents", 3).await?;
    store.finish_scope(lease, ScopeOutcome::Partial, 4).await?;
    Ok(())
}

async fn check(pool: &PgPool) -> TestResult {
    let fields: (bool, i64) =
        sqlx::query_as("SELECT f_1,f_2 FROM inventory.kind_0 WHERE id='shared'")
            .fetch_one(pool)
            .await?;
    assert_eq!(fields, (true, 7));
    let facets: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory.resource_facets WHERE id='shared'")
            .fetch_one(pool)
            .await?;
    assert_eq!(facets, 2);
    let parents: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM inventory.resource_parents WHERE child_id='shared'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(parents, 2);
    Ok(())
}
