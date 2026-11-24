use super::super::barrier_proxy::BarrierProxy;
use super::*;
use inventory_engine::sql::{Bounds, ReaderUrl, SqlValue, run as run_sql};
use sqlx::{Connection, PgConnection};
use std::time::Duration;

const WRITER_PAUSE_KEY: i64 = 73_148_592;
const READER_BOUND: Duration = Duration::from_secs(2);

pub(super) async fn run(
    environment: &Environment,
    oracle: &TypedSeedOracle,
    expected: &InventorySnapshot,
) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database =
        Database::create(environment, directory.path().join("sql-writing.sqlite")).await?;
    let Database::Postgres(postgres) = &database else {
        anyhow::bail!("SQL writer concurrency requires PostgreSQL");
    };
    let mut owner = PgConnection::connect(postgres.url()).await?;
    let role = format!("sql_reader_{}", uuid::Uuid::new_v4().simple());
    let created = sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'inventory-sql-test' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
    ))).execute(&mut owner).await;
    let result = match created {
        Ok(_) => {
            writing(
                environment,
                &database,
                &mut owner,
                postgres.url(),
                &role,
                oracle,
            )
            .await
        }
        Err(error) => Err(error.into()),
    };
    let cleaned = cleanup(&mut owner, &role).await;
    let closed = owner.close().await.map_err(Into::into);
    result.and(cleaned).and(closed)?;
    ensure!(
        &database_snapshot(&database, oracle).await? == expected,
        "sweep with concurrent readers differs from uninterrupted inventory"
    );
    Ok(())
}

async fn writing(
    environment: &Environment,
    database: &Database,
    owner: &mut PgConnection,
    owner_url: &str,
    role: &str,
    oracle: &TypedSeedOracle,
) -> Result<()> {
    let mut isolated = environment.clone();
    isolated.graph_name = format!("{}-sql-{}", environment.graph_name, uuid::Uuid::new_v4());
    let mut proxy = BarrierProxy::start(
        &environment.endpoint,
        &environment.ca_bundle,
        Duration::from_millis(100),
    )
    .await?;
    let endpoint_url = proxy.endpoint();
    let endpoint = EndpointOverride {
        url: &endpoint_url,
        ca_bundle: proxy.ca_bundle(),
    };
    let mut arguments = sweep_args_for(&isolated, database, 100, false, Some(endpoint))?;
    set_concurrency(&mut arguments, 1)?;
    let sweep = tokio::spawn(async move { run_cli(&isolated, &arguments).await });
    let result = observe_reads(&mut proxy, owner, owner_url, role).await;
    let unlocked = sqlx::query("SELECT pg_advisory_unlock_all()")
        .execute(&mut *owner)
        .await
        .map(|_| ())
        .map_err(Into::into);
    let released = proxy.release();
    let completed = sweep
        .await
        .context("joining concurrent sweep task")
        .and_then(|result| result)
        .and_then(|summary| assert_no_failures(&summary));
    let stopped = proxy.shutdown().await;
    result
        .and(unlocked)
        .and(released)
        .and(completed)
        .and(stopped)?;
    sql_reader::verify_vpc_ids(owner_url, role, oracle).await
}

async fn observe_reads(
    proxy: &mut BarrierProxy,
    owner: &mut PgConnection,
    owner_url: &str,
    role: &str,
) -> Result<()> {
    let checkpoint = proxy.wait_for_continuation().await?;
    ensure!(
        matches!(checkpoint.action, "DescribeVpcs" | "DescribeSubnets")
            && !checkpoint.token.is_empty(),
        "concurrent sweep must reach a native continuation request"
    );
    let expected: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.resources")
        .fetch_one(&mut *owner)
        .await?;
    ensure!(
        expected > 0,
        "first sweep page must be committed before reader checks"
    );
    let reader = prepare_reader(owner, owner_url, role).await?;
    install_writer_pause(owner).await?;
    proxy.release()?;
    let writer = observe_writer(owner).await?;
    for _ in 0..3 {
        let output = tokio::time::timeout(
            READER_BOUND,
            run_sql(
                &reader,
                "SELECT count(*) AS n FROM resources",
                Bounds {
                    millis: 1_000,
                    ..Bounds::default()
                },
            ),
        )
        .await
        .context("reader blocked behind the sweep's open page transaction")??;
        ensure!(
            output.rows == vec![vec![SqlValue::Int(expected)]],
            "reader must see exactly the committed pages, never the in-flight page"
        );
        ensure!(
            writer_blocked(owner, writer).await?,
            "real sweep writer must remain inside its uncommitted page during every read"
        );
    }
    Ok(())
}

async fn prepare_reader(owner: &mut PgConnection, url: &str, role: &str) -> Result<ReaderUrl> {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT USAGE ON SCHEMA inventory TO \"{role}\""
    )))
    .execute(&mut *owner)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "GRANT SELECT ON ALL TABLES IN SCHEMA inventory TO \"{role}\""
    )))
    .execute(&mut *owner)
    .await?;
    Ok(ReaderUrl::establish(&sql_reader::reader_url(url, role)?).await?)
}

async fn install_writer_pause(owner: &mut PgConnection) -> Result<()> {
    sqlx::query(
        "CREATE FUNCTION inventory.sql_reader_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM set_config('lock_timeout', '15s', true); PERFORM pg_advisory_xact_lock(73148592::bigint); RETURN NEW; END; $$"
    ).execute(&mut *owner).await?;
    sqlx::query(
        "CREATE TRIGGER sql_reader_pause AFTER INSERT ON inventory.resources FOR EACH ROW EXECUTE FUNCTION inventory.sql_reader_pause()"
    ).execute(&mut *owner).await?;
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(WRITER_PAUSE_KEY)
        .execute(&mut *owner)
        .await?;
    Ok(())
}

async fn observe_writer(owner: &mut PgConnection) -> Result<i32> {
    for _ in 0..500 {
        let pid: Option<i32> = sqlx::query_scalar(
            "SELECT a.pid FROM pg_stat_activity a JOIN pg_locks l ON l.pid=a.pid WHERE a.datname=current_database() AND a.state='active' AND a.xact_start IS NOT NULL AND a.wait_event='advisory' AND l.locktype='advisory' AND NOT l.granted AND l.objid::bigint=$1 LIMIT 1"
        ).bind(WRITER_PAUSE_KEY).fetch_optional(&mut *owner).await?;
        if let Some(pid) = pid {
            return Ok(pid);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    anyhow::bail!("sweep did not enter its open page transaction at the writer pause")
}

async fn writer_blocked(owner: &mut PgConnection, pid: i32) -> Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity a JOIN pg_locks l ON l.pid=a.pid WHERE a.pid=$1 AND a.xact_start IS NOT NULL AND a.state='active' AND a.wait_event='advisory' AND l.locktype='advisory' AND NOT l.granted AND l.objid::bigint=$2)"
    ).bind(pid).bind(WRITER_PAUSE_KEY).fetch_one(&mut *owner).await?)
}

async fn cleanup(owner: &mut PgConnection, role: &str) -> Result<()> {
    let trigger = sqlx::query("DROP TRIGGER IF EXISTS sql_reader_pause ON inventory.resources")
        .execute(&mut *owner)
        .await
        .map(|_| ())
        .map_err(Into::into);
    let function = sqlx::query("DROP FUNCTION IF EXISTS inventory.sql_reader_pause()")
        .execute(&mut *owner)
        .await
        .map(|_| ())
        .map_err(Into::into);
    let owned = sqlx::query(sqlx::AssertSqlSafe(format!("DROP OWNED BY \"{role}\"")))
        .execute(&mut *owner)
        .await
        .map(|_| ())
        .map_err(Into::into);
    let dropped = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP ROLE IF EXISTS \"{role}\""
    )))
    .execute(&mut *owner)
    .await
    .map(|_| ())
    .map_err(Into::into);
    trigger.and(function).and(owned).and(dropped)
}
