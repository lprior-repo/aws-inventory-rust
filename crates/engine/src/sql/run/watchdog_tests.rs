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
#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_caller_dropped_when_query_sleeping_then_reader_backend_stops_before_statement_timeout()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let role = format!("{}_cancel", database.name());
    let result = dropped_query_case(&pool, database.url(), &role).await;
    pool.close().await;
    let deleted = database.finish().await;
    let dropped = drop_role(&role).await;
    deleted?;
    dropped?;
    result
}

async fn dropped_query_case(pool: &PgPool, owner_url: &str, role: &str) -> TestResult {
    sqlx::query("CREATE SCHEMA inventory").execute(pool).await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'cancel-reader' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
    )))
    .execute(pool)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA inventory TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    let reader_url = PgConnectOptions::from_str(owner_url)?
        .username(role)
        .password("cancel-reader")
        .to_url_lossy()
        .to_string();
    let reader = ReaderUrl::establish(&reader_url).await?;
    let task = tokio::spawn(async move {
        run(&reader, "SELECT pg_sleep(60)::text", Bounds::default()).await
    });
    let sleeping = wait_for_sleeping_backend(pool, role).await;
    task.abort();
    let joined = task.await;
    let stopped = match &sleeping {
        Ok(backend) => wait_for_backend_exit(pool, backend).await,
        Err(_) => Ok(()),
    };
    sleeping?;
    stopped?;
    if !matches!(&joined, Err(error) if error.is_cancelled()) {
        return Err(format!("query task was not cancelled: {joined:?}").into());
    }
    Ok(())
}

struct SleepingBackend {
    pid: i32,
    age_secs: f64,
    seen_at: std::time::Instant,
}

async fn wait_for_sleeping_backend(pool: &PgPool, role: &str) -> TestResult<SleepingBackend> {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let before = std::time::Instant::now();
        let row: Option<(i32, f64)> = sqlx::query_as(
            "SELECT pid, EXTRACT(EPOCH FROM clock_timestamp() - query_start)::float8 \
             FROM pg_stat_activity WHERE usename = $1 AND state = 'active' \
             AND query LIKE 'SELECT pg_sleep%' AND pid <> pg_backend_pid() LIMIT 1",
        )
        .bind(role)
        .fetch_optional(pool)
        .await?;
        if let Some((pid, age_secs)) = row {
            return Ok(SleepingBackend {
                pid,
                age_secs,
                seen_at: before,
            });
        }
        if std::time::Instant::now() >= deadline {
            return Err("reader backend never began executing pg_sleep".into());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn wait_for_backend_exit(pool: &PgPool, backend: &SleepingBackend) -> TestResult {
    let budget = Bounds::default().millis as f64 / 1000.0;
    loop {
        let present: i64 =
            sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE pid = $1")
                .bind(backend.pid)
                .fetch_one(pool)
                .await?;
        let elapsed = backend.age_secs + backend.seen_at.elapsed().as_secs_f64();
        if present == 0 {
            return if elapsed < budget {
                Ok(())
            } else {
                Err(format!("reader backend exited at {elapsed:.2}s").into())
            };
        }
        if elapsed >= budget {
            return Err(format!("reader backend still present at {elapsed:.2}s").into());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
