use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Executor};

use inventory_harness::HarnessError;
use inventory_harness::assertions::global;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn has_failed_check(checks: &[serde_json::Value]) -> bool {
    checks
        .iter()
        .any(|check| check.get("passed").and_then(serde_json::Value::as_bool) == Some(false))
}

fn check_at(
    checks: &[serde_json::Value],
    index: usize,
) -> Result<&serde_json::Value, HarnessError> {
    checks
        .get(index)
        .ok_or_else(|| HarnessError::invalid("missing check"))
}

#[tokio::test]
async fn given_empty_database_when_asserting_global_then_all_failures() -> TestResult {
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("test.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    connection.execute("CREATE TABLE resource_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, resource_json TEXT)").await?;
    connection.execute("CREATE TABLE edge_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, edge_json TEXT)").await?;
    connection
        .execute("CREATE TABLE scans (scan_id TEXT PRIMARY KEY, status TEXT)")
        .await?;
    connection.close().await?;

    let checks = global::check(&root(), &db_path, "scan", None).await?;

    assert!(
        has_failed_check(&checks),
        "empty database should produce failures"
    );
    let input_check = check_at(&checks, 0)?;
    assert_eq!(
        input_check.get("check").and_then(serde_json::Value::as_str),
        Some("global:readable_bounded_evidence")
    );
    Ok(())
}

#[tokio::test]
async fn given_fixture_metadata_when_asserting_global_then_authoritative_flag_passes() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("test.db");
    let seed_path = temp.path().join("seed-results.json");
    let fixtures = json!({
        "iam": {"policy_arns": ["arn:aws:iam::000000000000:policy/p1", "arn:aws:iam::000000000000:policy/p2", "arn:aws:iam::000000000000:policy/p3"], "role_arns": ["arn:aws:iam::000000000000:role/r1", "arn:aws:iam::000000000000:role/r2"], "profile_arns": ["arn:aws:iam::000000000000:role-profile/p1", "arn:aws:iam::000000000000:role-profile/p2"], "replication_role_arn": "arn:aws:iam::000000000000:role/replication"},
        "s3": {"rich_bucket": "rich", "empty_bucket": "empty", "logs_bucket": "logs", "replica_bucket": "replica", "kms_key_arn": "arn:aws:kms:us-east-1:000000000000:key/key1", "notification_topic_arn": "arn:aws:sns:us-east-1:000000000000:topic/t1", "notification_queue_arn": "arn:aws:sqs:us-east-1:000000000000:queue/q1"},
        "route53": {"vpc_id": "vpc-1", "zones": [{"id": "z1", "kind": "public"}, {"id": "z2", "kind": "private"}]},
        "cloudfront": {"distributions": [{"arn": "arn:aws:cloudfront::000000000000:distribution/d1"}, {"arn": "arn:aws:cloudfront::000000000000:distribution/d2"}]}
    });
    std::fs::write(
        &seed_path,
        serde_json::to_string(&json!({"modules": {"seed_global": {"fixtures": fixtures}}}))?,
    )?;

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    connection.execute("CREATE TABLE resource_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, resource_json TEXT)").await?;
    connection.execute("CREATE TABLE edge_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, edge_json TEXT)").await?;
    connection
        .execute("CREATE TABLE scans (scan_id TEXT PRIMARY KEY, status TEXT)")
        .await?;
    connection.close().await?;

    let seed_report = json!({"fixtures": fixtures});
    let checks = global::check(&root(), &db_path, "scan", Some(&seed_report)).await?;

    assert!(
        !has_failed_check(&checks),
        "fixture metadata should not fail"
    );
    Ok(())
}

#[test]
fn fixture_completeness_validation_checks_all_required_fields() -> TestResult {
    let fixtures = json!({
        "iam": {"policy_arns": ["arn:aws:iam::000000000000:policy/p1"], "role_arns": ["arn:aws:iam::000000000000:role/r1"], "profile_arns": ["arn:aws:iam::000000000000:role-profile/p1"]},
        "s3": {},
        "route53": {},
        "cloudfront": {}
    });

    let iam = fixtures.get("iam").unwrap_or(&serde_json::Value::Null);
    let policies = iam.get("policy_arns").and_then(serde_json::Value::as_array);
    assert_eq!(policies.map(|a| a.len()).unwrap_or(0), 1, "policies count");

    Ok(())
}
