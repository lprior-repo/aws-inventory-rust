use serde_json::Value;
use sqlx::Connection;

use inventory_harness::assertions::cloud_control;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check_at(checks: &[Value], index: usize) -> Result<&Value, Box<dyn std::error::Error>> {
    checks.get(index).ok_or_else(|| "missing check".into())
}

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn given_pinned_fixtures_when_selected_then_exactly_thirteen_authored_cases_remain() -> TestResult {
    let (fixtures, digest) = cloud_control::golden_fixtures(&root())?;
    assert_eq!(fixtures.len(), 13);
    assert_eq!(
        digest,
        "5736639a04254e70b1830dc35d1a5399bb10f75bfe5344775951569f524432ba"
    );
    assert!(
        fixtures
            .iter()
            .all(|row| !row.to_string().contains("111111111111"))
    );
    let document: Value =
        serde_json::to_value(toml::from_str::<toml::Table>(&std::fs::read_to_string(
            root().join("crates/aws/tests/fixtures/cloudcontrol-architecture.toml"),
        )?)?)?;
    let authored = document
        .get("fixtures")
        .and_then(Value::as_array)
        .map(Vec::len)
        .ok_or("authored fixtures")?;
    assert_eq!(authored, 18, "the file retains non-golden fixtures");
    assert!(fixtures.len() < authored, "selection actually filters");
    Ok(())
}

#[test]
fn given_explicit_identity_when_expected_then_the_authored_arn_is_used() -> TestResult {
    let (fixtures, _) = cloud_control::golden_fixtures(&root())?;
    let fixture = fixtures
        .iter()
        .find(|row| row.get("name").and_then(Value::as_str) == Some("sagemaker_model_role_and_vpc"))
        .ok_or("sagemaker fixture")?;
    assert_eq!(
        cloud_control::expected_id(fixture)?,
        "arn:aws:sagemaker:us-east-1:000000000000:model/model-demo"
    );
    Ok(())
}

#[test]
fn given_opaque_identity_when_expected_then_the_digest_identity_is_exact() -> TestResult {
    let (fixtures, _) = cloud_control::golden_fixtures(&root())?;
    let fixture = fixtures
        .iter()
        .find(|row| {
            row.get("name").and_then(Value::as_str)
                == Some("compound_attachment_is_not_a_fabricated_arn")
        })
        .ok_or("compound fixture")?;
    assert_eq!(
        cloud_control::expected_id(fixture)?,
        "awscc:aws:us-east-1:000000000000:AWS::EC2::VolumeAttachment:\
         2b59003faf30039a319046151da8979e1043bd79785fe86cb5a7af39c4f8f95e"
    );
    Ok(())
}

#[tokio::test]
async fn given_unreadable_evidence_when_asserting_then_one_bounded_failure_is_reported()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("empty.sqlite");
    std::fs::write(&database, b"")?;
    let checks = cloud_control::check(&root(), &database, "missing-scan", None).await?;
    assert_eq!(checks.len(), 1);
    let first = check_at(&checks, 0)?;
    assert_eq!(
        first.get("check"),
        Some(&Value::from("cloud_control:bounded_architecture_evidence"))
    );
    assert_eq!(first.get("passed"), Some(&Value::Bool(false)));
    assert!(
        first
            .get("detail")
            .and_then(Value::as_str)
            .is_some_and(|detail| !detail.is_empty())
    );
    Ok(())
}

#[tokio::test]
async fn given_no_rows_for_the_scan_when_asserting_then_every_case_reports_a_missing_read()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("empty-tables.sqlite");
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&database)
        .create_if_missing(true);
    let mut connection = sqlx::SqliteConnection::connect_with(&options).await?;
    for statement in [
        "CREATE TABLE resource_observations(scan_id TEXT,resource_json TEXT,source TEXT)",
        "CREATE TABLE edge_observations(scan_id TEXT,edge_json TEXT,source TEXT)",
    ] {
        sqlx::query(statement).execute(&mut connection).await?;
    }
    connection.close().await?;
    let checks = cloud_control::check(&root(), &database, "absent-scan", None).await?;
    assert_eq!(checks.len(), 14, "metadata plus one read per fixture");
    let first = check_at(&checks, 0)?;
    assert_eq!(
        first.get("check"),
        Some(&Value::from("cloud_control:architecture_fixture_metadata"))
    );
    let tail = checks.get(1..).ok_or("missing checks")?;
    assert!(tail.iter().all(|check| {
        check.get("passed") == Some(&Value::Bool(false))
            && check
                .get("check")
                .and_then(Value::as_str)
                .is_some_and(|name| name.ends_with(":one_typed_read"))
    }));
    Ok(())
}
