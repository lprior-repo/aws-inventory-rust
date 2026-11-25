use crate::sql::{Bounds, ReaderUrl, SqlError, SqlValue, run};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{ConnectOptions, Connection, PgConnection, PgPool, postgres::PgConnectOptions};
use std::{str::FromStr, time::Duration};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_disabled_server_budget_when_watchdog_expires_then_reader_backend_stops() -> TestResult
{
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let role = format!("{}_reader", database.name());
    let result = watchdog_case(&pool, database.url(), &role).await;
    pool.close().await;
    let deleted = database.finish().await;
    let dropped = drop_role(&role).await;
    deleted?;
    dropped?;
    let (error, active) = result?;
    assert_eq!(
        error.to_string(),
        "SQL connection or transaction step exceeded time bound"
    );
    assert!(matches!(error, SqlError::Timeout));
    assert!(!active, "watchdog must stop the reader's backend query");
    Ok(())
}

async fn watchdog_case(pool: &PgPool, owner_url: &str, role: &str) -> TestResult<(SqlError, bool)> {
    sqlx::query("CREATE SCHEMA inventory").execute(pool).await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'watchdog-reader' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
    ))).execute(pool).await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA inventory TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    let reader_url = PgConnectOptions::from_str(owner_url)?
        .username(role)
        .password("watchdog-reader")
        .to_url_lossy()
        .to_string();
    let mut reader = ReaderUrl::establish(&reader_url).await?;
    reader.server_millis = Some(0);
    let bounds = Bounds {
        millis: 200,
        ..Bounds::default()
    };
    let setting = run(
        &reader,
        "SELECT current_setting('statement_timeout') AS budget",
        bounds,
    )
    .await?;
    if setting.rows != vec![vec![SqlValue::Text("0".into())]] {
        return Err("test seam did not disable the server budget".into());
    }
    let (result, observed) = tokio::join!(
        run(&reader, "SELECT pg_sleep(60)::text", bounds),
        observe_sleep(pool, role),
    );
    observed?;
    let error = result.err().ok_or("expected an error")?;
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE usename=$1 AND state='active')",
    )
    .bind(role)
    .fetch_one(pool)
    .await?;
    Ok((error, active))
}

async fn observe_sleep(pool: &PgPool, role: &str) -> TestResult {
    for _ in 0..500 {
        let sleeping: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE usename=$1 AND state='active' AND wait_event='PgSleep')"
        ).bind(role).fetch_one(pool).await?;
        if sleeping {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err("reader backend never entered pg_sleep".into())
}

async fn drop_role(role: &str) -> TestResult {
    let mut admin = PgConnection::connect(&admin_url()?).await?;
    let result = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP ROLE IF EXISTS \"{role}\""
    )))
    .execute(&mut admin)
    .await;
    let closed = admin.close().await;
    result?;
    closed?;
    Ok(())
}
