use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};

use inventory_harness::emulator::Emulator;

use super::{emulator_binary, run, scan_pages, text, workspace};

type TestResult = Result<(), Box<dyn Error>>;

#[test]
#[ignore = "requires the pinned emulator binary and a collector build"]
fn given_owned_emulator_when_every_native_service_is_scanned_then_complete() -> TestResult {
    let work = tempfile::tempdir()?;
    let emulator = Emulator::start(&emulator_binary()?, work.path(), 900)?;
    let environment = emulator.environment();
    let database = text(&work.path().join("native.sqlite"))?;
    let report = scan(&database, &["--service", "all"], &environment)?;
    assert_terminal(&report, "native");
    assert_eq!(
        report.get("status").and_then(Value::as_str),
        Some("complete")
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the pinned emulator binary and a collector build"]
async fn given_owned_emulator_when_every_cloud_control_type_is_scanned_then_gaps_match_evidence()
-> TestResult {
    let work = tempfile::tempdir()?;
    let path = work.path().to_path_buf();
    let binary = emulator_binary()?;
    let emulator = tokio::task::spawn_blocking(move || Emulator::start(&binary, &path, 900))
        .await
        .map_err(|_| "emulator boot task panicked")??;
    let database = work.path().join("cloud.sqlite");
    let database_text = text(&database)?;
    let types = cloud_control_types()?;
    let report = scan(
        &database_text,
        &[
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
            &types,
        ],
        &emulator.environment(),
    )?;
    assert_terminal(&report, "cloud-control");

    let unsupported = unsupported_types(&database).await?;
    let recorded = recorded_gaps()?;
    assert_eq!(
        unsupported, recorded,
        "emulator unsupported types must match the committed capability matrix"
    );
    let status = report
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("absent");
    assert_eq!(
        status, "complete",
        "provider-unsupported types must not make the scan partial"
    );
    assert_eq!(
        report.pointer("/api/in_flight"),
        Some(&Value::from(0)),
        "cloud-control scan left work in flight"
    );
    Ok(())
}

fn number(block: &Value, key: &str) -> Result<u64, Box<dyn Error>> {
    block
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing universal counter {key}").into())
}

mod catalog;
mod combined;
mod universal;

use catalog::{cloud_control_types, recorded_gaps};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the pinned emulator binary and a collector build"]
async fn given_many_resources_when_scanned_with_small_pages_then_all_pages_are_pulled() -> TestResult
{
    let work = tempfile::tempdir()?;
    let path = work.path().to_path_buf();
    let binary = emulator_binary()?;
    let emulator = tokio::task::spawn_blocking(move || Emulator::start(&binary, &path, 900))
        .await
        .map_err(|_| "emulator boot task panicked")??;
    let environment = emulator.environment();
    let seeded = run(
        Command::new(env!("CARGO_BIN_EXE_inventory-harness")).args([
            "seed",
            "cloud-control",
            "--prefix",
            "inventory-page-topic",
            "--count",
            "6",
        ]),
        &environment,
    )?;
    assert!(
        seeded.status.success(),
        "Cloud Control seeding failed: {}",
        String::from_utf8_lossy(&seeded.stderr)
    );
    let seed_report: Value = serde_json::from_slice(&seeded.stdout)?;
    assert_eq!(
        seed_report
            .get("listed")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(6),
        "seeding must list exactly six topics: {seed_report}"
    );

    let database = work.path().join("pages.sqlite");
    let database_text = text(&database)?;
    let report = scan_pages(
        &database_text,
        &[
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
            "AWS::SNS::Topic",
        ],
        &environment,
        "10",
        "2",
    )?;
    assert_terminal(&report, "pagination");
    assert_eq!(
        report.get("status").and_then(Value::as_str),
        Some("complete")
    );
    assert_eq!(
        report
            .pointer("/scans/0/summary/resources")
            .and_then(Value::as_u64),
        Some(6),
        "pagination must pull every seeded topic"
    );

    let options = SqliteConnectOptions::new()
        .filename(&database)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let rows = sqlx::query(
        "SELECT page_number, next_token FROM collection_pages
         WHERE json_extract(scope_key, '$.collector') = 'cloud_control_list'
           AND json_extract(scope_key, '$.parent_id') = 'AWS::SNS::Topic'
         ORDER BY page_number",
    )
    .fetch_all(&mut connection)
    .await?;
    let mut pages = Vec::with_capacity(rows.len());
    for row in &rows {
        pages.push((
            row.try_get::<i64, _>(0)?,
            row.try_get::<Option<String>, _>(1)?,
        ));
    }

    let (observed, failures): (i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT count(*) FROM resource_observations
             WHERE scope_key IN (
               SELECT scope_key FROM collection_pages
               WHERE json_extract(scope_key, '$.collector') = 'cloud_control_get'
                 AND json_extract(scope_key, '$.parent_id') LIKE 'AWS::SNS::Topic:%')),
           (SELECT count(*) FROM collection_failures)",
    )
    .fetch_one(&mut connection)
    .await?;
    connection.close().await?;

    assert!(
        pages.len() >= 2,
        "page size two over six topics needs at least two list pages, got {}",
        pages.len()
    );
    for (index, (page_number, next_token)) in pages.iter().enumerate() {
        let ordinal = index.saturating_add(1);
        assert_eq!(
            *page_number,
            i64::try_from(ordinal)?,
            "list page numbers must be contiguous from one"
        );
        assert_eq!(
            next_token.is_none(),
            ordinal == pages.len(),
            "list page {page_number} must carry a continuation token unless it is the last"
        );
    }
    assert_eq!(
        observed, 6,
        "the list continuation chain must observe every seeded topic"
    );
    assert_eq!(failures, 0, "pagination reported a failure");
    Ok(())
}

fn assert_terminal(report: &Value, label: &str) {
    let status = report
        .pointer("/scans/0/status")
        .and_then(Value::as_str)
        .unwrap_or("absent");
    assert!(
        status == "complete" || status == "partial",
        "{label} scan is not terminal: {status}"
    );
    assert!(
        report.get("write_error").is_none_or(Value::is_null),
        "{label} scan reported a write error"
    );
    assert_eq!(
        report.pointer("/api/in_flight"),
        Some(&Value::from(0)),
        "{label} scan left work in flight"
    );
}

fn scan(
    database: &str,
    selection: &[&str],
    environment: &[(String, String)],
) -> Result<Value, Box<dyn Error>> {
    scan_pages(database, selection, environment, "2", "2")
}

async fn unsupported_types(database: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let options = SqliteConnectOptions::new()
        .filename(database)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let rows = sqlx::query(
        "SELECT scope_key, category, message FROM collection_failures ORDER BY scope_key",
    )
    .fetch_all(&mut connection)
    .await?;
    let mut unsupported = BTreeSet::new();
    for row in rows {
        let key: String = row.try_get(0)?;
        let category: String = row.try_get(1)?;
        let message: String = row.try_get(2)?;
        assert_eq!(category, "unsupported_type", "real failure: {message}");
        assert!(
            message.contains("TypeNotFoundException"),
            "unexpected unsupported failure: {message}"
        );
        let scope: Value = serde_json::from_str(&key)?;
        if let Some(parent) = scope.get("parent_id").and_then(Value::as_str) {
            unsupported.insert(parent.to_owned());
        }
    }
    connection.close().await?;
    Ok(unsupported)
}
