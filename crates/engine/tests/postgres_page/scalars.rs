use super::{TestResult, pages};
use inventory_core::record::{Exclusions, FieldId, Float, Time, Value};
use inventory_engine::inventory::{DatabaseUrl, PgStore, StoreError};
use sqlx::{PgPool, Row};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_all_scalars_when_page_commits_then_native_values_are_lossless() -> TestResult {
    pages::run(scalars).await
}

async fn scalars(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "scalars").await?;
    let observation = scalar_observation()?;
    let result = store
        .commit_page(pages::page(lease, vec![observation], 1, 2))
        .await?;
    assert_eq!(result.stats.resources_inserted, 1);
    let row = sqlx::query(
        "SELECT f_1,f_2,f_3,f_4,f_5,EXTRACT(epoch FROM f_6)::text AS seconds FROM inventory.kind_0",
    )
    .fetch_one(pool)
    .await?;
    check_scalars(&row)
}

fn scalar_observation() -> TestResult<inventory_core::inventory::Observation> {
    pages::observation(
        "scalar",
        [
            (1, Value::Bool(false)),
            (2, Value::Int(i64::MIN)),
            (3, Value::Float(Float::new(f64::NAN))),
            (5, Value::Bytes(vec![7; 1_048_576])),
            (6, Value::Time(Time::new(9_007_199_254, 740_993_000)?)),
        ],
        Exclusions::none(),
    )
}

fn check_scalars(row: &sqlx::postgres::PgRow) -> TestResult {
    assert!(!row.try_get::<bool, _>("f_1")?);
    assert_eq!(row.try_get::<i64, _>("f_2")?, i64::MIN);
    assert!(row.try_get::<f64, _>("f_3")?.is_nan());
    assert_eq!(row.try_get::<&str, _>("f_4")?, "scalar");
    let blob: Vec<u8> = row.try_get("f_5")?;
    assert_eq!(blob.len(), 1_048_576);
    assert!(blob.iter().all(|byte| *byte == 7));
    assert_eq!(row.try_get::<&str, _>("seconds")?, "9007199254.740993");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_excluded_time_when_raw_record_changes_then_hash_and_change_time_are_preserved()
-> TestResult {
    pages::run(excluded).await
}

async fn excluded(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "excluded").await?;
    store
        .commit_page(pages::page(lease, vec![timed(-1)?], 1, 2))
        .await?;
    let before = snapshot(pool).await?;
    let lease = pages::claim(store, "excluded", 3).await?;
    let result = store
        .commit_page(pages::page(lease, vec![timed(2)?], 2, 4))
        .await?;
    assert_eq!(result.stats, Default::default());
    let after = snapshot(pool).await?;
    assert_eq!(before.0, after.0);
    assert_eq!(before.1, after.1);
    assert!(after.2 > before.2);
    let seconds: String =
        sqlx::query_scalar("SELECT EXTRACT(epoch FROM f_6)::text FROM inventory.kind_0")
            .fetch_one(pool)
            .await?;
    assert_eq!(seconds, "2.000001");
    Ok(())
}

fn timed(seconds: i64) -> TestResult<inventory_core::inventory::Observation> {
    pages::observation(
        "timed",
        [(6, Value::Time(Time::new(seconds, 1000)?))],
        Exclusions::new(vec![vec![FieldId::new(6)?]])?,
    )
}

async fn snapshot(pool: &PgPool) -> TestResult<(Vec<u8>, i64, i64)> {
    Ok(sqlx::query_as("SELECT r.content_hash,r.last_changed,p.revision FROM inventory.resources r CROSS JOIN inventory.projection_outbox p")
        .fetch_one(pool)
        .await?)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_mismatched_scalar_when_page_commits_then_entire_page_is_rejected() -> TestResult {
    pages::run(mismatched).await
}

async fn mismatched(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let lease = pages::seed(store, "mismatched").await?;
    let valid = pages::observation("valid", [], Exclusions::none())?;
    let invalid = pages::observation("invalid", [(2, Value::Bool(true))], Exclusions::none())?;
    let result = store
        .commit_page(pages::page(lease, vec![valid, invalid], 1, 2))
        .await;
    assert!(matches!(
        result,
        Err(StoreError::Invalid(
            "record scalar does not match column schema"
        ))
    ));
    let counts: (i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.kind_0),(SELECT count(*) FROM inventory.resource_facets),(SELECT count(*) FROM inventory.projection_outbox)")
        .fetch_one(pool)
        .await?;
    assert_eq!(counts, (0, 0, 0, 0));
    Ok(())
}
