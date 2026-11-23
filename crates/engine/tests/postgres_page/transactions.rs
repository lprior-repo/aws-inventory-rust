use super::{TestResult, pages};
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};
use inventory_core::record::Exclusions;
use inventory_engine::inventory::{DatabaseUrl, PageCommit, PgStore, StoreError, WorkLease};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_late_queue_failure_when_page_commits_then_every_page_write_rolls_back() -> TestResult
{
    pages::run(rollback).await
}

async fn rollback(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "rollback").await?;
    sqlx::query(
        "ALTER TABLE inventory.work_queue ADD CONSTRAINT reject_page_token CHECK(token IS NULL)",
    )
    .execute(pool)
    .await?;
    let page = rollback_page(lease)?;
    let error = match store.commit_page(page).await {
        Err(error) => error,
        Ok(_) => return Err("constraint violation unexpectedly committed".into()),
    };
    assert!(error.to_string().contains("reject_page_token"));
    check_rollback(pool).await
}

fn rollback_page(lease: WorkLease) -> TestResult<PageCommit> {
    let observation = pages::observation("rollback", [], Exclusions::none())?;
    let mut page = pages::page(lease, vec![observation], 1, 2);
    page.tags.push(TagObservation::new(
        pages::key("rollback")?,
        TagCoverage::Complete,
        vec![Tag::new("a", "b")?],
    )?);
    page.children
        .push(pages::work("rollback", Some(pages::key("parent")?))?);
    Ok(page)
}

async fn check_rollback(pool: &PgPool) -> TestResult {
    let counts: (i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.kind_0),(SELECT count(*) FROM inventory.resource_facets),(SELECT count(*) FROM inventory.tags),(SELECT count(*) FROM inventory.tag_observations),(SELECT count(*) FROM inventory.work_tokens),(SELECT count(*) FROM inventory.projection_outbox)")
        .fetch_one(pool)
        .await?;
    assert_eq!(counts, (0, 0, 0, 0, 0, 0, 0));
    let queue: Vec<(i64, i64)> =
        sqlx::query_as("SELECT state,generation FROM inventory.work_queue")
            .fetch_all(pool)
            .await?;
    assert_eq!(queue, vec![(1, 1)]);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_repeated_digest_when_page_commits_then_scope_is_partial_and_stops() -> TestResult {
    pages::run(repeated).await
}

async fn repeated(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "repeated").await?;
    let observation = pages::observation("same", [], Exclusions::none())?;
    store
        .commit_page(pages::page(lease, vec![observation], 7, 2))
        .await?;
    let lease = pages::claim(store, "repeated", 3).await?;
    let observation = pages::observation("same", [], Exclusions::none())?;
    let result = store
        .commit_page(pages::page(lease, vec![observation], 7, 4))
        .await?;
    assert!(result.repeated_token);
    assert_eq!(result.next_generation, None);
    assert_eq!(result.stats, Default::default());
    let outcome: String = sqlx::query_scalar("SELECT outcome FROM inventory.scope_outcomes")
        .fetch_one(pool)
        .await?;
    assert_eq!(outcome, "partial");
    let queue: (i64, bool, i64) =
        sqlx::query_as("SELECT state,token IS NULL,lease_until FROM inventory.work_queue")
            .fetch_one(pool)
            .await?;
    assert_eq!(queue, (2, true, 0));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_expired_generation_when_page_commits_then_stale_writer_cannot_mutate_data()
-> TestResult {
    pages::run(stale).await
}

async fn stale(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let expired = pages::seed(store, "stale").await?;
    let current = pages::claim(store, "stale", 10_002).await?;
    let observation = pages::observation("stale", [], Exclusions::none())?;
    assert!(matches!(
        store
            .commit_page(pages::page(expired, vec![observation], 1, 10_003))
            .await,
        Err(StoreError::StaleLease)
    ));
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.resources")
        .fetch_one(pool)
        .await?;
    assert_eq!(count, 0);
    let observation = pages::observation("current", [], Exclusions::none())?;
    let result = store
        .commit_page(pages::page(current, vec![observation], 2, 10_003))
        .await?;
    assert_eq!(result.stats.resources_inserted, 1);
    Ok(())
}
