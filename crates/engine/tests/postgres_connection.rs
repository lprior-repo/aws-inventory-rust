mod postgres_support;

use inventory_engine::inventory::StoreError;
use inventory_engine::inventory::postgres::{DatabaseUrl, PgStore, PgWriteConfig};
use inventory_harness::postgres::{TestDatabase, admin_url};
use postgres_support::{TestResult, registry};
use sqlx::PgPool;

#[test]
fn postgres_given_password_when_rendered_then_url_is_redacted() -> TestResult {
    let url = DatabaseUrl::parse("postgresql://user:s%40cret@localhost/inventory?password=hidden")?;
    assert_eq!(url.to_string(), "postgresql://***");
    assert!(!format!("{url:?}").contains("s%40cret"));
    assert!(!format!("{url:?}").contains("hidden"));
    let invalid = DatabaseUrl::parse("postgres://user:secret@localhost:invalid/db");
    assert!(invalid.is_err());
    assert!(!format!("{invalid:?}").contains("secret"));
    assert!(DatabaseUrl::parse("sqlite://secret").is_err());
    assert!(DatabaseUrl::parse(&"p".repeat(8_193)).is_err());
    assert_eq!(PgWriteConfig::default().max_connections(), 8);
    assert!(PgWriteConfig::new(0).is_err());
    assert!(PgWriteConfig::new(65).is_err());
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_catalog_when_created_then_reopens() -> TestResult {
    with_database(check_reopen).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_empty_database_when_opened_then_refused() -> TestResult {
    with_database(check_empty).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_wrong_version_when_opened_then_refused() -> TestResult {
    with_database(check_version).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_wrong_digest_when_opened_then_refused() -> TestResult {
    with_database(check_digest).await
}

async fn with_database(check: impl AsyncFn(&DatabaseUrl, &PgPool) -> TestResult) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let pool = PgPool::connect(database.url()).await?;
    let result = check(&url, &pool).await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check_reopen(url: &DatabaseUrl, _: &PgPool) -> TestResult {
    let (registry, bytes) = registry(1, None)?;
    let expected = *registry.digest();
    let store = PgStore::create(url, registry, PgWriteConfig::default()).await?;
    store.validate_catalog(&bytes).await?;
    assert!(matches!(
        store.validate_catalog(b"wrong").await,
        Err(StoreError::CatalogMismatch)
    ));
    store.close().await;
    let reopened = PgStore::open(url, PgWriteConfig::default()).await?;
    assert_eq!(*reopened.registry().digest(), expected);
    assert_eq!(reopened.config().max_connections(), 8);
    reopened.close().await;
    Ok(())
}

async fn check_empty(url: &DatabaseUrl, _: &PgPool) -> TestResult {
    assert!(PgStore::open(url, PgWriteConfig::default()).await.is_err());
    Ok(())
}

async fn create(url: &DatabaseUrl) -> TestResult {
    let (registry, _) = registry(1, None)?;
    PgStore::create(url, registry, PgWriteConfig::default())
        .await?
        .close()
        .await;
    Ok(())
}

async fn check_version(url: &DatabaseUrl, pool: &PgPool) -> TestResult {
    create(url).await?;
    sqlx::query("UPDATE inventory.metadata SET value=$1 WHERE key='schema_version'")
        .bind(b"99".as_slice())
        .execute(pool)
        .await?;
    assert!(matches!(
        PgStore::open(url, PgWriteConfig::default()).await,
        Err(StoreError::SchemaVersion { found: 99 })
    ));
    Ok(())
}

async fn check_digest(url: &DatabaseUrl, pool: &PgPool) -> TestResult {
    create(url).await?;
    sqlx::query("UPDATE inventory.metadata SET value=$1 WHERE key='catalog_digest'")
        .bind(vec![0_u8; 32])
        .execute(pool)
        .await?;
    assert!(matches!(
        PgStore::open(url, PgWriteConfig::default()).await,
        Err(StoreError::CatalogMismatch)
    ));
    Ok(())
}

#[tokio::test]
async fn postgres_given_connection_error_when_reported_then_password_is_absent() -> TestResult {
    let directory = tempfile::tempdir()?;
    let address = format!(
        "postgresql://user:secret@localhost/db?host={}",
        directory.path().display()
    );
    let url = DatabaseUrl::parse(&address)?;
    let error = match PgStore::open(&url, PgWriteConfig::default()).await {
        Err(error) => error,
        Ok(store) => {
            store.close().await;
            return Err("unexpected PostgreSQL server".into());
        }
    };
    assert!(!error.to_string().contains("secret"));
    assert!(!format!("{error:?}").contains("secret"));
    Ok(())
}
