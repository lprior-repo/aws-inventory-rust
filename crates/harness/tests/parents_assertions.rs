//! Negative controls for the parent architecture assertion oracle.

use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Executor};

use inventory_harness::assertions::parents;
use inventory_harness::HarnessError;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn has_failed_check(checks: &[serde_json::Value]) -> bool {
    checks.iter().any(|check| {
        check
            .get("passed")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    })
}

#[test]
fn fixture_loading_validates_byte_bound() -> TestResult {
    // Load the real fixture and verify it's under the byte bound
    let path = root().join("crates/aws/tests/fixtures/cloudcontrol-parents.json");
    let bytes = std::fs::read(&path)?;
    assert!(
        bytes.len() <= 1_048_576,
        "fixture exceeds 1 MiB byte bound"
    );
    Ok(())
}

#[test]
fn fixture_loading_validates_record_counts() -> TestResult {
    let (document, _) = parents::seed::load_fixtures(&root())?;
    let parents = document
        .get("parents")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("missing parents"))?;
    let children = document
        .get("children")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("missing children"))?;
    let pages = document
        .get("pagination_children")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("missing pagination_children"))?;

    assert_eq!(parents.len(), 12, "parent fixture count");
    assert_eq!(children.len(), 72, "child fixture count");
    assert_eq!(pages.len(), 98, "pagination fixture count");
    Ok(())
}

#[test]
fn fixture_loading_validates_unique_parent_keys() -> TestResult {
    let (document, _) = parents::seed::load_fixtures(&root())?;
    let parents = document
        .get("parents")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("missing parents"))?;

    let keys: std::collections::HashSet<String> = parents
        .iter()
        .filter_map(|p| p.get("key").and_then(serde_json::Value::as_str).map(str::to_owned))
        .collect();
    assert_eq!(keys.len(), 12, "unique parent keys");
    Ok(())
}

#[test]
fn fixture_loading_validates_child_type_coverage() -> TestResult {
    let (document, _) = parents::seed::load_fixtures(&root())?;
    let children = document
        .get("children")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("missing children"))?;

    let types: std::collections::HashSet<String> = children
        .iter()
        .filter_map(|c| c.get("type_name").and_then(serde_json::Value::as_str).map(str::to_owned))
        .collect();
    assert_eq!(types.len(), 24, "child-type coverage");
    Ok(())
}

#[tokio::test]
async fn given_empty_database_when_asserting_parents_then_failures() -> TestResult {
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("test.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    connection.execute("CREATE TABLE resource_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, resource_json TEXT)").await?;
    connection.execute("CREATE TABLE edge_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, edge_json TEXT)").await?;
    connection.execute("CREATE TABLE scans (scan_id TEXT PRIMARY KEY, status TEXT)").await?;
    connection.close().await?;

    let seed_report = json!({"fixture_sha256": "wrong", "status": "incomplete"});
    let checks = parents::check(&root(), &db_path, "scan", Some(&seed_report)).await?;

    assert!(has_failed_check(&checks), "empty database should produce failures");
    Ok(())
}

#[test]
fn normalized_identity_uses_expected_id_when_present() -> TestResult {
    let child = json!({"expected_resource_id": "arn:aws:fake:us-east-1:000000000000:resource/test"});
    let seed = json!({});
    let id = parents::identity::normalized_id(&child, &seed);
    assert_eq!(
        id,
        Some("arn:aws:fake:us-east-1:000000000000:resource/test".to_owned())
    );
    Ok(())
}

#[test]
fn normalized_identity_computes_hash_when_no_expected_id() -> TestResult {
    let child = json!({"type_name": "AWS::Test::Type"});
    let seed = json!({"identifier": "my-unique-identifier"});
    let id = parents::identity::normalized_id(&child, &seed);
    assert!(id.is_some());
    let computed = id.unwrap();
    assert!(computed.starts_with("awscc:aws:us-east-1:000000000000:AWS::Test::Type:"));
    assert_eq!(computed.len(), 55 + 64, "computed identity length");
    Ok(())
}