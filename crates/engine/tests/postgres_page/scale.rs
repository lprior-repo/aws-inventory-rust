use super::{TestResult, pages};
use inventory_core::inventory::Observation;
use inventory_core::record::Exclusions;
use inventory_engine::inventory::{DatabaseUrl, PgStore};
use sqlx::PgPool;
use std::time::Instant;

type Versions = (Vec<String>, Vec<String>, Vec<String>, Vec<String>);

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and preloaded pg_stat_statements"]
async fn given_fifty_thousand_unchanged_resources_when_page_commits_then_three_statements_and_no_content_writes()
-> TestResult {
    pages::run(unchanged).await
}

async fn unchanged(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let before = initial(store, pool).await?;
    let lease = pages::claim(store, "scale", 3).await?;
    let page = pages::page(lease, observations()?, 2, 4);
    let calls_before = calls(pool).await?;
    let start = Instant::now();
    let result = store.commit_page(page).await?;
    let elapsed = start.elapsed();
    let statements = calls(pool)
        .await?
        .checked_sub(calls_before)
        .ok_or("statement count underflow")?;
    assert_eq!(statements, 3);
    assert_eq!(result.stats, Default::default());
    assert_eq!(versions(pool).await?, before);
    assert_counts(pool).await?;
    println!(
        "P9 rows=50000 unchanged_statements={statements} elapsed_ms={}",
        elapsed.as_millis()
    );
    Ok(())
}

async fn initial(store: &PgStore, pool: &PgPool) -> TestResult<Versions> {
    sqlx::query("CREATE EXTENSION pg_stat_statements")
        .execute(pool)
        .await?;
    let lease = pages::seed(store, "scale").await?;
    let first = store
        .commit_page(pages::page(lease, observations()?, 1, 2))
        .await?;
    assert_eq!(first.stats.resources_inserted, 50_000);
    versions(pool).await
}

fn observations() -> TestResult<Vec<Observation>> {
    (0..50_000)
        .map(|index| pages::observation(&format!("resource_{index:05}"), [], Exclusions::none()))
        .collect()
}

async fn calls(pool: &PgPool) -> TestResult<i64> {
    Ok(sqlx::query_scalar("SELECT COALESCE(sum(calls),0)::bigint FROM pg_stat_statements WHERE dbid=(SELECT oid FROM pg_database WHERE datname=current_database()) AND query NOT IN('BEGIN','COMMIT','ROLLBACK') AND query NOT LIKE '%pg_stat_statements%'")
        .fetch_one(pool)
        .await?)
}

async fn versions(pool: &PgPool) -> TestResult<Versions> {
    Ok(sqlx::query_as("SELECT (SELECT array_agg(DISTINCT xmin::text) FROM inventory.resources),(SELECT array_agg(DISTINCT xmin::text) FROM inventory.kind_0),(SELECT array_agg(DISTINCT xmin::text) FROM inventory.resource_facets),(SELECT array_agg(DISTINCT xmin::text) FROM inventory.projection_outbox)")
        .fetch_one(pool)
        .await?)
}

async fn assert_counts(pool: &PgPool) -> TestResult {
    let counts: (i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.kind_0),(SELECT count(*) FROM inventory.resource_facets),(SELECT count(*) FROM inventory.resource_scopes)")
        .fetch_one(pool)
        .await?;
    assert_eq!(counts, (50_000, 50_000, 50_000, 50_000));
    Ok(())
}
