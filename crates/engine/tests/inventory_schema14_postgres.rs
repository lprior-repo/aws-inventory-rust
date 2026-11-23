mod postgres_support;

use inventory_engine::inventory::{SchemaRegistry, StoreError, postgres::schema};
use inventory_harness::postgres::{TestDatabase, admin_url};
use postgres_support::{TestResult, registry};
use sqlx::{PgPool, Row};
use std::time::Instant;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_201_kinds_when_initialized_then_registry_and_200_table_batches_round_trip()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let result = check_schema(&pool).await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check_schema(pool: &PgPool) -> TestResult {
    let (registry, _) = registry(201, None)?;
    let started = Instant::now();
    schema::initialize(pool, &registry).await?;
    let create_ms = started.elapsed().as_millis();
    let started = Instant::now();
    let loaded = schema::load_registry(pool).await?;
    let reopen_ms = started.elapsed().as_millis();
    assert_registry(&loaded, &registry);
    let groups: Vec<i64> = sqlx::query_scalar("SELECT count(*) FROM pg_class WHERE relnamespace='inventory'::regnamespace AND relkind='r' AND relname ~ '^kind_[0-9]+$' GROUP BY xmin::text ORDER BY count(*)")
        .fetch_all(pool)
        .await?;
    assert_eq!(groups, [1, 200]);
    assert!(schema::initialize(pool, &registry).await.is_err());
    record_budget(pool, create_ms, reopen_ms).await?;
    round_trip_scalars(pool).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_identifier_boundaries_when_initialized_then_63_is_preserved_and_64_refused()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let result = check_identifier_boundaries(&pool).await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check_identifier_boundaries(pool: &PgPool) -> TestResult {
    let name = "t".repeat(63);
    let (valid, _) = registry(1, Some(&name))?;
    schema::initialize(pool, &valid).await?;
    let actual: String = sqlx::query_scalar("SELECT table_name FROM inventory.kinds")
        .fetch_one(pool)
        .await?;
    assert_eq!(actual, name);
    sqlx::query("DROP SCHEMA inventory CASCADE")
        .execute(pool)
        .await?;
    let (invalid, _) = registry(1, Some(&"t".repeat(64)))?;
    assert!(matches!(
        schema::initialize(pool, &invalid).await,
        Err(StoreError::Invalid("PostgreSQL identifier exceeds bounds"))
    ));
    Ok(())
}

async fn record_budget(pool: &PgPool, create_ms: u128, reopen_ms: u128) -> TestResult {
    let row = sqlx::query("SELECT (SELECT count(*) FROM information_schema.tables WHERE table_schema='inventory') AS tables,(SELECT count(*) FROM information_schema.columns WHERE table_schema='inventory') AS columns,pg_database_size(current_database()) AS bytes")
        .fetch_one(pool).await?;
    let tables: i64 = row.try_get("tables")?;
    let columns: i64 = row.try_get("columns")?;
    let bytes: i64 = row.try_get("bytes")?;
    println!(
        "P11 synthetic_kinds=201 create_ms={create_ms} reopen_ms={reopen_ms} tables={tables} columns={columns} empty_database_bytes={bytes}"
    );
    Ok(())
}

fn assert_registry(loaded: &SchemaRegistry, expected: &SchemaRegistry) {
    assert_eq!(loaded.digest(), expected.digest());
    assert_eq!(loaded.schemas().len(), expected.schemas().len());
    assert!(
        loaded
            .schemas()
            .iter()
            .zip(expected.schemas())
            .all(|(left, right)| {
                left.kind() == right.kind()
                    && left.table() == right.table()
                    && left.columns().len() == right.columns().len()
                    && left
                        .columns()
                        .iter()
                        .zip(right.columns())
                        .all(|(left, right)| {
                            left.path() == right.path() && left.scalar() == right.scalar()
                        })
            })
    );
}

async fn round_trip_scalars(pool: &PgPool) -> TestResult {
    sqlx::query("INSERT INTO inventory.kind_0(account,region,id,content_hash,first_seen,last_changed,record,f_1,f_2,f_3,f_4,f_5,f_6) VALUES('a','r','i',$1,1,2,$1,true,$2,'NaN'::float8,'text',$3,timestamptz '2000-01-01 00:00:00+00' + (123456789::bigint / 1000) * interval '1 microsecond')")
        .bind(vec![1_u8]).bind(i64::MIN).bind(vec![7_u8; 1_048_576]).execute(pool).await?;
    let row = sqlx::query("SELECT f_1,f_2,f_3,f_4,f_5,to_char(f_6 AT TIME ZONE 'UTC','YYYY-MM-DD HH24:MI:SS.US') AS time FROM inventory.kind_0")
        .fetch_one(pool).await?;
    assert!(row.try_get::<bool, _>("f_1")?);
    assert_eq!(row.try_get::<i64, _>("f_2")?, i64::MIN);
    assert!(row.try_get::<f64, _>("f_3")?.is_nan());
    assert_eq!(row.try_get::<String, _>("f_4")?, "text");
    assert_eq!(row.try_get::<Vec<u8>, _>("f_5")?, vec![7_u8; 1_048_576]);
    assert_eq!(
        row.try_get::<String, _>("time")?,
        "2000-01-01 00:00:00.123456"
    );
    Ok(())
}
