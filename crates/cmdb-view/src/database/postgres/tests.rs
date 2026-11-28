mod batching;

#[path = "../../../../engine/tests/postgres_support/mod.rs"]
mod catalog;

use super::*;
use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig};
use inventory_harness::postgres::{TestDatabase, admin_url};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn database() -> TestResult<TestDatabase> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let (registry, _) = catalog::registry(1, None)?;
    let store = PgStore::create(
        &DatabaseUrl::parse(database.url())?,
        registry,
        PgWriteConfig::default(),
    )
    .await?;
    store.close().await;
    Ok(database)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_reader_pool_when_all_connections_are_used_then_settings_and_writes_are_read_only()
-> TestResult {
    let database = database().await?;
    let reader = Database::open(database.url()).await?;
    let mut connections = Vec::with_capacity(4);
    for _ in 0..4 {
        let mut connection = reader.pool.acquire().await?;
        let settings: (String, String) = sqlx::query_as("SELECT current_setting('search_path'),current_setting('default_transaction_read_only')")
            .fetch_one(&mut *connection).await?;
        assert_eq!(settings, ("inventory".to_owned(), "on".to_owned()));
        connections.push(connection);
    }
    let connection = connections.first_mut().ok_or("missing reader connection")?;
    let error = sqlx::query(
        "INSERT INTO inventory.metadata(key,value) VALUES('reader-write',decode('00','hex'))",
    )
    .execute(&mut **connection)
    .await
    .err()
    .ok_or("expected read-only error")?;
    let error = error.as_database_error().ok_or("expected database error")?;
    assert_eq!(error.code().as_deref(), Some("25006"));
    drop(connections);
    let mut snapshot = reader.snapshot().await?;
    let settings: (String, String) = sqlx::query_as(
        "SELECT current_setting('transaction_isolation'),current_setting('transaction_read_only')",
    )
    .fetch_one(&mut *snapshot.tx)
    .await?;
    assert_eq!(settings, ("repeatable read".to_owned(), "on".to_owned()));
    snapshot.finish().await?;
    reader.pool.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_incompatible_schema_when_opening_reader_then_version_fails_closed()
-> TestResult {
    let database = database().await?;
    let writer = PgPool::connect(database.url()).await?;
    sqlx::query("UPDATE inventory.metadata SET value=$1 WHERE key='schema_version'")
        .bind(b"999".as_slice())
        .execute(&writer)
        .await?;
    let error = Database::open(database.url())
        .await
        .err()
        .ok_or("expected version error")?;
    assert_eq!(
        error.to_string(),
        "unsupported PostgreSQL inventory schema; expected 14 or 15"
    );
    sqlx::query("DELETE FROM inventory.metadata WHERE key='schema_version'")
        .execute(&writer)
        .await?;
    let error = Database::open(database.url())
        .await
        .err()
        .ok_or("expected absent version error")?;
    assert_eq!(
        error.to_string(),
        "unsupported PostgreSQL inventory schema; expected 14 or 15"
    );
    writer.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
async fn postgres_given_unparseable_connection_url_when_opening_reader_then_password_is_not_exposed()
-> TestResult {
    let target = "postgres://reader:secret-password@localhost:not-a-port/inventory";
    let error = crate::Database::open(target)
        .await
        .err()
        .ok_or("expected connection error")?;
    assert_eq!(error.to_string(), "PostgreSQL connection failed");
    assert_eq!(format!("{error:?}"), "PostgresConnect");
    Ok(())
}

#[tokio::test]
async fn postgres_given_closed_port_when_opening_reader_then_password_is_not_exposed() -> TestResult
{
    let socket = tokio::net::TcpSocket::new_v4()?;
    socket.bind("127.0.0.1:0".parse()?)?;
    let port = socket.local_addr()?.port();
    let target = format!("postgres://reader:secret-password@127.0.0.1:{port}/inventory");
    let error = crate::Database::open(&target)
        .await
        .err()
        .ok_or("expected connection error")?;
    assert_eq!(error.to_string(), "PostgreSQL connection failed");
    assert_eq!(format!("{error:?}"), "PostgresConnect");
    assert!(!error.to_string().contains("secret-password"));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_mixed_case_schemes_when_opening_then_dispatch_uses_postgres() -> TestResult
{
    let database = database().await?;
    let (_, authority) = database
        .url()
        .split_once("://")
        .ok_or("missing URL scheme")?;
    for scheme in ["postgres", "postgresql", "POSTGRES", "PoStGrEsQl"] {
        let reader = crate::Database::open(&format!("{scheme}://{authority}")).await?;
        assert_eq!(reader.counts().await?, DatabaseCounts::default());
        assert!(crate::is_postgres_url(&format!("{scheme}://{authority}")));
    }
    assert!(!crate::is_postgres_url("inventory.sqlite"));
    assert!(!crate::is_postgres_url("postgres-not-a-url.sqlite"));
    assert!(!crate::is_postgres_url("mysql://localhost/inventory"));
    database.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_absent_inventory_schema_when_opening_then_schema_diagnostic_is_redacted()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let connection = PgPool::connect(database.url()).await?;
    let value: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&connection)
        .await?;
    assert_eq!(value, 1);
    let error = sqlx::query("SELECT value FROM inventory.metadata WHERE key='schema_version'")
        .fetch_optional(&connection)
        .await
        .err()
        .ok_or("expected missing metadata table")?;
    assert_eq!(
        error
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref(),
        Some("42P01")
    );
    connection.close().await;
    probes::capture_pool();
    let result = crate::Database::open(database.url()).await;
    let pool = probes::take_pool().ok_or("expected the production reader pool")?;
    assert!(
        pool.is_closed(),
        "metadata failure must close its reader pool"
    );
    let error = result
        .err()
        .ok_or("expected missing inventory schema to fail")?;
    assert_eq!(error.to_string(), "PostgreSQL inventory schema is missing");
    assert_eq!(format!("{error:?}"), "PostgresSchema");
    database.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_four_reader_leases_when_acquiring_a_fifth_then_it_waits_for_capacity()
-> TestResult {
    let database = database().await?;
    let reader = Database::open(database.url()).await?;
    assert_eq!(reader.pool.options().get_max_connections(), 4);
    let mut leases = Vec::with_capacity(4);
    for _ in 0..4 {
        leases.push(reader.pool.acquire().await?);
    }
    let _: tokio::time::error::Elapsed =
        tokio::time::timeout(std::time::Duration::from_millis(100), reader.pool.acquire())
            .await
            .err()
            .ok_or("expected a fifth lease to wait for capacity")?;
    drop(leases);
    let mut lease = reader.pool.acquire().await?;
    let value: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&mut *lease)
        .await?;
    assert_eq!(value, 1);
    drop(lease);
    reader.pool.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_missing_database_when_opening_then_database_diagnostic_is_redacted()
-> TestResult {
    use sqlx::ConnectOptions;
    let database = TestDatabase::create(&admin_url()?).await?;
    let target = database
        .url()
        .parse::<sqlx::postgres::PgConnectOptions>()?
        .database(&format!("{}_absent", database.name()))
        .to_url_lossy();
    let direct_error = PgPool::connect(target.as_str())
        .await
        .err()
        .ok_or("expected absent database")?;
    assert_eq!(
        direct_error
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref(),
        Some("3D000")
    );
    let error = crate::Database::open(target.as_str())
        .await
        .err()
        .ok_or("expected database error")?;
    database.finish().await?;
    assert_eq!(error.to_string(), "PostgreSQL database does not exist");
    assert_eq!(format!("{error:?}"), "PostgresDatabase");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_non_login_role_when_opening_then_authentication_diagnostic_is_redacted()
-> TestResult {
    use sqlx::ConnectOptions;
    let database = TestDatabase::create(&admin_url()?).await?;
    let administrator = PgPool::connect(database.url()).await?;
    let role = SqlName::parse(&format!("{}_role", database.name()))?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE {} NOLOGIN",
        role.as_str()
    )))
    .execute(&administrator)
    .await?;
    let target = database
        .url()
        .parse::<sqlx::postgres::PgConnectOptions>()?
        .username(role.as_str())
        .password("never-display-this-password")
        .to_url_lossy();
    let direct_result = PgPool::connect(target.as_str()).await;
    let result = crate::Database::open(target.as_str()).await;
    sqlx::query(sqlx::AssertSqlSafe(format!("DROP ROLE {}", role.as_str())))
        .execute(&administrator)
        .await?;
    administrator.close().await;
    database.finish().await?;
    let direct_error = direct_result.err().ok_or("expected authentication error")?;
    assert_eq!(
        direct_error
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref(),
        Some("28000")
    );
    let error = result.err().ok_or("expected authentication diagnostic")?;
    assert_eq!(error.to_string(), "PostgreSQL authentication failed");
    assert_eq!(format!("{error:?}"), "PostgresAuthentication");
    Ok(())
}
