//! R07: an interrupted PostgreSQL initialization is distinguishable from a ready
//! schema and `create` resumes it instead of failing forever.
mod postgres_support;

use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig, StoreError};
use inventory_harness::postgres::{TestDatabase, admin_url};
use postgres_support::{TestResult, registry};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_interrupted_batch_when_created_again_then_initialization_resumes() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let result = check(&pool, &DatabaseUrl::parse(database.url())?).await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check(pool: &PgPool, url: &DatabaseUrl) -> TestResult {
    let config = PgWriteConfig::default();
    // Fail the last table, the only one in the second 200-table batch, after the
    // first batch has committed.
    let (registry, _) = registry(201, None)?;
    let Some(last) = registry.schemas().last() else {
        return Err("registry is empty".into());
    };
    let last = format!("inventory.{}", last.table().as_str());
    sqlx::query("CREATE TABLE public.fail_target(name text NOT NULL)")
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO public.fail_target(name) VALUES($1)")
        .bind(last)
        .execute(pool)
        .await?;
    sqlx::query("CREATE FUNCTION fail_last() RETURNS event_trigger LANGUAGE plpgsql AS $$ DECLARE r record; BEGIN FOR r IN SELECT * FROM pg_event_trigger_ddl_commands() LOOP IF r.object_identity IN (SELECT name FROM public.fail_target) THEN RAISE EXCEPTION 'injected interruption'; END IF; END LOOP; END $$")
        .execute(pool).await?;
    sqlx::query("CREATE EVENT TRIGGER fail_last ON ddl_command_end EXECUTE FUNCTION fail_last()")
        .execute(pool)
        .await?;

    assert!(PgStore::create(url, registry, config).await.is_err());

    assert!(!PgStore::is_ready(url, config).await?);
    assert!(matches!(
        PgStore::open(url, config).await,
        Err(StoreError::InitializationIncomplete)
    ));
    let created: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.kind_schemas")
        .fetch_one(pool)
        .await?;
    assert_eq!(created, 200);

    let (other, _) = registry_with_other_digest()?;
    assert!(matches!(
        PgStore::create(url, other, config).await,
        Err(StoreError::CatalogMismatch)
    ));

    sqlx::query("DROP EVENT TRIGGER fail_last")
        .execute(pool)
        .await?;
    let (registry, _) = self::registry(201, None)?;
    PgStore::create(url, registry, config).await?.close().await;

    assert!(PgStore::is_ready(url, config).await?);
    let reopened = PgStore::open(url, config).await?;
    assert_eq!(reopened.registry().schemas().len(), 201);
    reopened.close().await;
    let (registry, _) = self::registry(201, None)?;
    assert!(matches!(
        PgStore::create(url, registry, config).await,
        Err(StoreError::Invalid("inventory schema already exists"))
    ));
    Ok(())
}

fn registry_with_other_digest() -> TestResult<(inventory_engine::inventory::SchemaRegistry, Vec<u8>)>
{
    registry(2, None)
}
