use super::{close, ReaderUrl};
use crate::sql::{Bounds, SqlError, run};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{
    ConnectOptions, Connection, PgConnection, PgPool, postgres::PgConnectOptions,
};
use std::{str::FromStr, time::Duration};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_pending_server_query_when_closing_then_shutdown_does_not_drain_results() -> TestResult
{
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let result = close_pending(database.url(), &pool).await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn close_pending(url: &str, pool: &PgPool) -> TestResult {
    let mut connection = PgConnection::connect(url).await?;
    sqlx::query("SET statement_timeout = 0")
        .execute(&mut connection)
        .await?;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut connection)
        .await?;
    {
        let sleeping = sqlx::query("SELECT pg_sleep(60)").execute(&mut connection);
        tokio::pin!(sleeping);
        tokio::select! {
            result = &mut sleeping => {
                result?;
                return Err("sleep completed before connection shutdown".into());
            }
            observed = observe_sleep(pool, pid) => observed?,
        }
    }
    close(connection).await?;
    let sleeping: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND state='active' AND wait_event='PgSleep')",
    ).bind(pid).fetch_one(pool).await?;
    assert!(
        sleeping,
        "shutdown must not wait for the server query to finish"
    );
    Ok(())
}

async fn observe_sleep(pool: &PgPool, pid: i32) -> TestResult {
    for _ in 0..500 {
        let sleeping: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND state='active' AND wait_event='PgSleep')",
        ).bind(pid).fetch_one(pool).await?;
        if sleeping {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err("server did not enter pg_sleep".into())
}

#[tokio::test]
async fn given_peer_stalling_output_when_closing_then_step_timeout_is_returned() -> TestResult {
    let mut peer = crate::sql::test_peer::Peer::start(crate::sql::test_peer::Stall::Close).await?;
    let result = tokio::time::timeout(Duration::from_secs(30), close_stalled(&mut peer)).await;
    let cleanup = peer.finish().await;
    let error = result??.err().ok_or("expected an error")?;
    cleanup?;
    assert_eq!(
        error.to_string(),
        "SQL connection or transaction step exceeded time bound"
    );
    assert!(matches!(error, crate::sql::SqlError::Timeout));
    Ok(())
}

async fn close_stalled(
    peer: &mut crate::sql::test_peer::Peer,
) -> Result<Result<(), crate::sql::SqlError>, Box<dyn std::error::Error>> {
    let mut connection = PgConnection::connect_with(&peer.options).await?;
    sqlx::raw_sql("BEGIN TRANSACTION READ ONLY")
        .execute(&mut connection)
        .await?;
    let mut sql = String::from("SELECT 1");
    sql.try_reserve_exact(8_388_608)?;
    sql.extend(std::iter::repeat_n(' ', 8_388_608));
    {
        let sending = sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).execute(&mut connection);
        tokio::pin!(sending);
        tokio::select! {
            result = &mut sending => {
                result?;
                return Err("query completed before output backpressure".into());
            }
            observed = &mut peer.stalled => observed?,
        }
    }
    Ok(close(connection).await)
}
#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_only_reader_when_querying_then_namespace_and_write_gate_apply() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let pool = PgPool::connect(database.url()).await?;
    let role = format!("{}_native_reader", database.name());
    let result = native_reader_case(&pool, database.url(), &role).await;
    pool.close().await;
    let deleted = database.finish().await;
    let dropped = drop_native_reader_role(&role).await;
    deleted?;
    dropped?;
    result
}

async fn native_reader_case(pool: &PgPool, owner_url: &str, role: &str) -> TestResult {
    sqlx::query("CREATE SCHEMA native_scan").execute(pool).await?;
    sqlx::query("CREATE TABLE native_scan.items (id integer PRIMARY KEY)")
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO native_scan.items VALUES (7)")
        .execute(pool)
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'native-reader' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
    )))
    .execute(pool)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA native_scan TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT SELECT ON native_scan.items TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    let url = PgConnectOptions::from_str(owner_url)?
        .username(role)
        .password("native-reader")
        .to_url_lossy()
        .to_string();
    let reader = ReaderUrl::establish_native(&url).await?;
    let result = run(&reader, "SELECT id FROM items", Bounds::default()).await?;
    if result.columns.len() != 1
        || result.columns.first().map(String::as_str) != Some("id")
        || result.rows.len() != 1
    {
        return Err(format!("unexpected native reader result: {result:?}").into());
    }
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT INSERT ON native_scan.items TO \"{role}\""
    )))
    .execute(pool)
    .await?;
    if !matches!(ReaderUrl::establish_native(&url).await, Err(SqlError::ReaderRequired)) {
        return Err("native reader accepted an inventory write grant".into());
    }
    Ok(())
}

async fn drop_native_reader_role(role: &str) -> TestResult {
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
