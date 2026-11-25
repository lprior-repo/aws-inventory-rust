#[path = "../crates/engine/tests/postgres_support/mod.rs"]
mod postgres_support;
#[path = "../crates/engine/tests/sql_support/mod.rs"]
mod sql_support;

use postgres_support::TestResult;
use sql_support::Fixture;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_reader_when_invoking_sql_then_table_and_row_limit_are_observable() -> TestResult {
    sql_support::run(cli_table).await
}

async fn cli_table(fixture: &Fixture) -> TestResult {
    sqlx::query("INSERT INTO inventory.resources VALUES('a','r','k','page-1',NULL,'\\x00',1,1)")
        .execute(&fixture.pool)
        .await?;
    assert_eq!(
        inventory_engine::sql::run(
            &fixture.reader,
            "SELECT id FROM resources",
            inventory_engine::sql::Bounds::default(),
        )
        .await?
        .rows,
        vec![vec![inventory_engine::sql::SqlValue::Text("page-1".into())]]
    );
    let output = command(
        &fixture.reader_url,
        "SELECT 12::int8 AS n, true AS flag FROM resources",
        "2",
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "n  | flag\n-- | ----\n12 | true\n"
    );
    let refused = command(
        &fixture.reader_url,
        "SELECT n FROM generate_series(1,2) n",
        "1",
    )
    .await?;
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(refused.stderr)?,
        "aws-inventory: SQL result exceeds rows bound\n"
    );
    let owner = command(&fixture.owner_url, "SELECT 1", "1").await?;
    assert_eq!(owner.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(owner.stderr)?,
        "aws-inventory: SQL requires a reader role with no inventory write or administrative privileges\n"
    );
    Ok(())
}

async fn command(url: &str, sql: &str, limit: &str) -> TestResult<std::process::Output> {
    Ok(
        tokio::process::Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
            .args(["sql", "--database", url, "--limit", limit, sql])
            .env("RUST_LOG", "off")
            .output()
            .await?,
    )
}

#[test]
fn given_disallowed_statement_when_invoking_sql_then_validation_precedes_connection() -> TestResult
{
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
        .args([
            "sql",
            "--database",
            "postgres://unreachable.invalid/inventory",
            "DELETE FROM resources",
        ])
        .env("RUST_LOG", "off")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "aws-inventory: SQL denied: only SELECT, WITH, VALUES, EXPLAIN, TABLE or SHOW queries are permitted\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_invalid_row_limits_when_invoking_sql_then_bounds_precede_connection() -> TestResult {
    for limit in ["0", "10001"] {
        let output = command(
            "postgres://unreachable.invalid/inventory",
            "SELECT 1",
            limit,
        )
        .await?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(String::from_utf8(output.stdout)?, "");
        assert_eq!(
            String::from_utf8(output.stderr)?,
            "aws-inventory: SQL bounds must be positive and no greater than the defaults\n"
        );
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_float4_when_invoking_sql_then_the_table_shows_its_shortest_decimal() -> TestResult {
    sql_support::run(cli_float4).await
}

async fn cli_float4(fixture: &Fixture) -> TestResult {
    for (sql, expected) in [
        ("SELECT 1.1::float4 AS n", "n  \n---\n1.1\n"),
        ("SELECT 'NaN'::float4 AS n", "n  \n---\nNaN\n"),
    ] {
        let output = command(&fixture.reader_url, sql, "1").await?;
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(String::from_utf8(output.stdout)?, expected);
        assert_eq!(String::from_utf8(output.stderr)?, "");
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_expanding_text_when_invoking_sql_then_escaped_output_stays_byte_bounded()
-> TestResult {
    sql_support::run(cli_expanding_text).await
}

async fn cli_expanding_text(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT repeat(chr(8203), 1048576) AS n",
        "SELECT repeat(chr(1), 1677722) AS n",
    ] {
        let output = command(&fixture.reader_url, sql, "1").await?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(String::from_utf8(output.stdout)?, "");
        assert_eq!(
            String::from_utf8(output.stderr)?,
            "aws-inventory: SQL result exceeds bytes bound\n"
        );
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_advisory_named_columns_when_invoking_sql_then_harmless_reads_succeed() -> TestResult
{
    sql_support::run(cli_advisory_columns).await
}

async fn cli_advisory_columns(fixture: &Fixture) -> TestResult {
    for sql in [
        "SELECT pg_advisory_note AS n FROM (VALUES (1)) AS r(pg_advisory_note)",
        "SELECT \"pg_advisory_note\" AS n FROM (VALUES (1)) AS r(\"pg_advisory_note\")",
    ] {
        let output = command(&fixture.reader_url, sql, "1").await?;
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(String::from_utf8(output.stdout)?, "n\n-\n1\n");
        assert_eq!(String::from_utf8(output.stderr)?, "");
    }
    Ok(())
}
