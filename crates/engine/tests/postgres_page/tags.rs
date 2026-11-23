use super::{TestResult, pages};
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};
use inventory_core::record::Exclusions;
use inventory_engine::inventory::{DatabaseUrl, PgStore};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_complete_tags_when_later_coverage_is_incomplete_then_contents_are_retained()
-> TestResult {
    pages::run(retention).await
}

async fn retention(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "tags").await?;
    let observation = pages::observation("tagged", [], Exclusions::none())?;
    store
        .commit_page(pages::page(lease, vec![observation], 1, 2))
        .await?;
    observe(store, TagCoverage::Complete, complete()?, 2, 4).await?;
    let revision = revision(pool).await?;
    incomplete(store, pool, revision).await?;
    observe(store, TagCoverage::Complete, complete()?, 6, 12).await?;
    check(pool, "complete", 12, 12, revision).await?;
    observe(store, TagCoverage::Complete, Vec::new(), 7, 14).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.tags")
        .fetch_one(pool)
        .await?;
    assert_eq!(count, 0);
    assert!(self::revision(pool).await? > revision);
    Ok(())
}

async fn incomplete(store: &PgStore, pool: &PgPool, revision: i64) -> TestResult {
    observe(
        store,
        TagCoverage::Partial,
        vec![Tag::new("Name", "partial")?],
        3,
        6,
    )
    .await?;
    check(pool, "partial", 6, 4, revision).await?;
    observe(store, TagCoverage::Failed, Vec::new(), 4, 8).await?;
    check(pool, "failed", 8, 4, revision).await?;
    observe(store, TagCoverage::Unobserved, Vec::new(), 5, 10).await?;
    check(pool, "unobserved", 10, 4, revision).await
}

fn complete() -> TestResult<Vec<Tag>> {
    Ok(vec![Tag::new("Name", "one")?, Tag::new("env", "prod")?])
}

async fn observe(
    store: &PgStore,
    coverage: TagCoverage,
    tags: Vec<Tag>,
    token: u8,
    at: i64,
) -> TestResult {
    let lease = pages::claim(store, "tags", at.checked_sub(1).ok_or("time underflow")?).await?;
    let mut page = pages::page(lease, Vec::new(), token, at);
    page.tags
        .push(TagObservation::new(pages::key("tagged")?, coverage, tags)?);
    let result = store.commit_page(page).await?;
    assert_eq!(result.stats, Default::default());
    Ok(())
}

async fn revision(pool: &PgPool) -> TestResult<i64> {
    Ok(
        sqlx::query_scalar("SELECT revision FROM inventory.projection_outbox")
            .fetch_one(pool)
            .await?,
    )
}

async fn check(
    pool: &PgPool,
    coverage: &str,
    at: i64,
    complete_at: i64,
    revision: i64,
) -> TestResult {
    let actual: (String, i64, i64) = sqlx::query_as(
        "SELECT coverage,observed_at,last_complete_at FROM inventory.tag_observations",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(actual, (coverage.to_owned(), at, complete_at));
    let tags = stored_tags(pool).await?;
    assert_eq!(
        tags,
        vec![("Name".into(), "one".into()), ("env".into(), "prod".into())]
    );
    assert_eq!(self::revision(pool).await?, revision);
    Ok(())
}

async fn stored_tags(pool: &PgPool) -> TestResult<Vec<(String, String)>> {
    Ok(sqlx::query_as(
        "SELECT tag_key,tag_value FROM inventory.tags ORDER BY tag_key COLLATE \"C\"",
    )
    .fetch_all(pool)
    .await?)
}
