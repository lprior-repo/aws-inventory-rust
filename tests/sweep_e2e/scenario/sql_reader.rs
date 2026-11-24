use super::*;
use inventory_engine::sql::{Bounds, ReaderUrl, SqlValue, run};
use sqlx::{ConnectOptions, Connection, PgConnection, postgres::PgConnectOptions};
use std::str::FromStr;

pub(super) async fn verify_committed_checkpoint(database: &Database) -> Result<()> {
    let Database::Postgres(database) = database else {
        anyhow::bail!("SQL reader verification requires PostgreSQL");
    };
    let mut owner = PgConnection::connect(database.url()).await?;
    let role = format!("sql_reader_{}", uuid::Uuid::new_v4().simple());
    let created = sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE \"{role}\" LOGIN PASSWORD 'inventory-sql-test' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS")))
        .execute(&mut owner).await;
    let result = match created {
        Ok(_) => {
            let result = read_committed(&mut owner, database.url(), &role).await;
            let owned = sqlx::query(sqlx::AssertSqlSafe(format!("DROP OWNED BY \"{role}\"")))
                .execute(&mut owner)
                .await
                .map(|_| ())
                .map_err(Into::into);
            let role = sqlx::query(sqlx::AssertSqlSafe(format!("DROP ROLE \"{role}\"")))
                .execute(&mut owner)
                .await
                .map(|_| ())
                .map_err(Into::into);
            result.and(owned).and(role)
        }
        Err(error) => Err(error.into()),
    };
    let closed = owner.close().await.map_err(Into::into);
    result.and(closed)
}

async fn read_committed(owner: &mut PgConnection, url: &str, role: &str) -> Result<()> {
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
    let expected: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.resources")
        .fetch_one(&mut *owner)
        .await?;
    ensure!(
        expected > 0,
        "the paused sweep checkpoint must contain committed resources"
    );
    let reader_url = reader_url(url, role)?;
    let reader = ReaderUrl::establish(&reader_url).await?;
    let rows = run(
        &reader,
        "SELECT count(*) AS n FROM resources",
        Bounds::default(),
    )
    .await?;
    ensure!(
        rows.rows == vec![vec![SqlValue::Int(expected)]],
        "the reader must see all resources committed at the paused checkpoint"
    );
    verify_cli(&reader_url, expected).await
}

pub(super) fn reader_url(url: &str, role: &str) -> Result<String> {
    Ok(PgConnectOptions::from_str(url)?
        .username(role)
        .password("inventory-sql-test")
        .to_url_lossy()
        .to_string())
}

pub(super) async fn verify_vpc_ids(url: &str, role: &str, oracle: &TypedSeedOracle) -> Result<()> {
    let query = "SELECT id FROM aws_ec2_vpc ORDER BY id";
    let text = run_cli_sql(&reader_url(url, role)?, query).await?;
    let mut lines = text.lines().map(str::trim);
    ensure!(lines.next() == Some("id"), "missing typed VPC ID header");
    let divider = lines.next().context("missing typed VPC ID divider")?;
    ensure!(
        !divider.is_empty() && divider.bytes().all(|byte| byte == b'-'),
        "invalid typed VPC ID divider: {divider:?}"
    );
    let ids: std::collections::BTreeSet<_> = lines.collect();
    for pair in &oracle.pairs {
        ensure!(
            ids.contains(pair.vpc_id.as_str()),
            "SQL CLI did not display seeded VPC {}",
            pair.vpc_id.as_str()
        );
    }
    Ok(())
}

async fn verify_cli(reader_url: &str, expected: i64) -> Result<()> {
    let text = run_cli_sql(reader_url, "SELECT count(*) AS n FROM resources").await?;
    ensure!(
        text.lines().map(str::trim).collect::<Vec<_>>()
            == vec![
                "n".to_owned(),
                "-".repeat(expected.to_string().len()),
                expected.to_string()
            ],
        "SQL CLI must display the committed checkpoint count"
    );
    Ok(())
}

async fn run_cli_sql(reader_url: &str, statement: &str) -> Result<String> {
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
        .args(["sql", "--database", reader_url, statement])
        .env("RUST_LOG", "off")
        .output()
        .await?;
    ensure!(
        output.status.code() == Some(0),
        "SQL CLI failed: {:?}",
        output
    );
    ensure!(
        String::from_utf8(output.stderr)?.is_empty(),
        "SQL CLI wrote stderr"
    );
    Ok(String::from_utf8(output.stdout)?)
}

#[tokio::test]
async fn given_file_database_when_verifying_sql_then_backend_mismatch_is_an_error() -> Result<()> {
    let database = Database::File(std::path::PathBuf::from("unused.sqlite"));
    assert_eq!(
        verify_committed_checkpoint(&database)
            .await
            .err()
            .ok_or_else(|| anyhow::anyhow!("expected an error"))?
            .to_string(),
        "SQL reader verification requires PostgreSQL"
    );
    Ok(())
}
