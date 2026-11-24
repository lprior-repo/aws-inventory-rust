use super::close;
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{Connection, PgConnection, PgPool};
use std::time::Duration;

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
