//! Operation coverage contracts: join identity, exclusions, determinism, drift.
use inventory_harness::HarnessError;
use inventory_harness::audit::{operation_coverage, run_operation_coverage};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[path = "operations/real_workspace.rs"]
mod real_workspace;
#[path = "operations/reviewed_additions.rs"]
mod reviewed_additions;
#[path = "operations/unavailable_callsite_scan_refuses_to_publish_or_check.rs"]
mod unavailable_callsite_scan_refuses_to_publish_or_check;
#[path = "operations/version_fixture.rs"]
mod version_fixture;
#[path = "operations/version_provenance.rs"]
mod version_provenance;

type BoxError = Box<dyn std::error::Error>;
type TestResult = Result<(), BoxError>;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pointer<'a>(value: &'a Value, path: &str) -> Result<&'a Value, BoxError> {
    value
        .pointer(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}

fn number(value: &Value, path: &str) -> Result<u64, BoxError> {
    pointer(value, path)?
        .as_u64()
        .ok_or_else(|| BoxError::from(format!("{path} is not an integer")))
}

fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, BoxError> {
    pointer(value, path)?
        .as_array()
        .ok_or_else(|| BoxError::from(format!("{path} is not an array")))
}

fn fixture() -> Result<tempfile::TempDir, BoxError> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    std::fs::create_dir_all(root.join("catalog"))?;
    std::fs::create_dir_all(root.join("crates/aws/src"))?;
    std::fs::create_dir_all(root.join("docs"))?;
    let crates = vec![
        json!({"name": "aws-sdk-alpha", "operations": [
            {"module": "list_things", "api_operation": "ListThings"},
            {"module": "alpha_only", "api_operation": "AlphaOnly"},
        ]}),
        json!({"name": "aws-sdk-beta", "operations": [
            {"module": "beta_only", "api_operation": "BetaOnly"},
            {"module": "list_things", "api_operation": "ListThings"},
        ]}),
        json!({"name": "aws-sdk-cloudcontrol", "operations": [
            {"module": "delete_resource", "api_operation": "DeleteResource"},
            {"module": "get_resource", "api_operation": "GetResource"},
            {"module": "list_resources", "api_operation": "ListResources"},
        ]}),
    ];
    let inventory = json!({
        "schema_version": 1,
        "summary": {"crates": 3, "operations": 7},
        "crates": crates,
    });
    std::fs::write(
        root.join("catalog/aws-sdk-operations.json"),
        serde_json::to_string_pretty(&inventory)?,
    )?;
    let matrix = json!({
        "schema_version": 1,
        "emulator_run": {"evidence_directory": "artifacts/emulator-fixture"},
        "operations": [
            {"id": "alpha.list_things", "sdk_crate": "aws-sdk-alpha",
             "sdk_operation": "list_things",
             "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect"}],
             "emulator_coverage": {"status": "collector_success"}},
            {"id": "alpha.alpha_only", "sdk_crate": "aws-sdk-alpha",
             "sdk_operation": "alpha_only",
             "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect"}],
             "emulator_coverage": {"status": "sdk_decode_failure"}},
            {"id": "beta.list_things", "sdk_crate": "aws-sdk-beta",
             "sdk_operation": "list_things",
             "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect"}],
             "emulator_coverage": {"status": "explicit_sdk_probe_success"}},
            {"id": "cloudcontrol.list_resources", "sdk_crate": "aws-sdk-cloudcontrol",
             "sdk_operation": "list_resources",
             "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect"}],
             "emulator_coverage": {"status": "collector_success"}},
            {"id": "cloudcontrol.get_resource", "sdk_crate": "aws-sdk-cloudcontrol",
             "sdk_operation": "get_resource",
             "source_calls": [{"path": "crates/aws/src/collect.rs", "function": "collect"}],
             "emulator_coverage": {"status": "emulator_not_implemented"}},
        ],
    });
    std::fs::write(
        root.join("catalog/aws-operations.json"),
        serde_json::to_string_pretty(&matrix)?,
    )?;
    std::fs::write(
        root.join("crates/aws/src/collect.rs"),
        r"
fn collect(
    alpha: &aws_sdk_alpha::Client,
    beta: &aws_sdk_beta::Client,
    cloud: &aws_sdk_cloudcontrol::Client,
) {
    alpha.list_things().send();
    alpha.alpha_only().send();
    beta.list_things().send();
    cloud.list_resources().send();
    cloud.list_resources().send();
    cloud.get_resource().send();
}
",
    )?;
    Ok(directory)
}

#[test]
fn coverage_joins_on_crate_and_module_and_excludes_decode_failures() -> TestResult {
    let directory = fixture()?;
    let report = operation_coverage(directory.path())?.to_value()?;
    assert_eq!(number(&report, "/summary/operations_total")?, 7);
    assert_eq!(number(&report, "/summary/crates_total")?, 3);
    assert_eq!(number(&report, "/summary/services_total")?, 3);
    assert_eq!(number(&report, "/summary/exercised")?, 4);
    assert_eq!(number(&report, "/summary/unexercised")?, 3);
    assert_eq!(
        number(&report, "/summary/provider_generic_operations_total")?,
        2
    );
    assert_eq!(number(&report, "/summary/provider_generic_exercised")?, 2);
    assert_eq!(number(&report, "/summary/typed_native_exercised")?, 2);
    assert_eq!(
        number(&report, "/summary/services_with_exercised_operations")?,
        3
    );

    let exercised = array(&report, "/exercised_operations")?;
    assert_eq!(exercised.len(), 4);
    let list_things: Vec<&Value> = exercised
        .iter()
        .filter(|row| row.get("module").and_then(Value::as_str) == Some("list_things"))
        .collect();
    assert_eq!(list_things.len(), 2);
    assert_ne!(
        list_things.first().and_then(|row| row.get("service")),
        list_things.last().and_then(|row| row.get("service"))
    );
    assert_eq!(
        list_things
            .iter()
            .filter(|row| row.get("provider_generic").and_then(Value::as_bool) == Some(true))
            .count(),
        0
    );
    let cloud: Vec<&Value> = exercised
        .iter()
        .filter(|row| row.get("service").and_then(Value::as_str) == Some("cloudcontrol"))
        .collect();
    assert_eq!(cloud.len(), 2, "Cloud Control counts once per operation");
    assert!(
        cloud
            .iter()
            .all(|row| row.get("provider_generic").and_then(Value::as_bool) == Some(true))
    );

    let excluded = array(&report, "/excluded_operations")?;
    assert_eq!(excluded.len(), 1);
    assert_eq!(
        excluded
            .first()
            .and_then(|row| row.get("module"))
            .and_then(Value::as_str),
        Some("alpha_only")
    );
    assert_eq!(
        excluded
            .first()
            .and_then(|row| row.get("status"))
            .and_then(Value::as_str),
        Some("sdk_decode_failure")
    );

    let sample = array(&report, "/unexercised_sample")?;
    assert_eq!(sample.len(), 3);
    assert_eq!(
        pointer(&report, "/unexercised_sample_truncated")?.as_bool(),
        Some(false)
    );
    assert_eq!(
        pointer(&report, "/status_source")?,
        &json!("operation_matrix")
    );
    let services = array(&report, "/by_service")?;
    let alpha = services
        .iter()
        .find(|row| row.get("service").and_then(Value::as_str) == Some("alpha"))
        .ok_or("alpha service")?;
    assert_eq!(alpha.get("total").and_then(Value::as_u64), Some(2));
    assert_eq!(alpha.get("exercised").and_then(Value::as_u64), Some(1));
    let first = services
        .first()
        .and_then(|row| row.get("service"))
        .and_then(Value::as_str);
    assert_eq!(first, Some("alpha"), "services are sorted");
    Ok(())
}

#[test]
fn coverage_output_is_deterministic_and_sorted() -> TestResult {
    let directory = fixture()?;
    let first = operation_coverage(directory.path())?.to_value()?;
    let second = operation_coverage(directory.path())?.to_value()?;
    assert_eq!(first, second);
    let services: Vec<&str> = array(&first, "/by_service")?
        .iter()
        .filter_map(|row| row.get("service").and_then(Value::as_str))
        .collect();
    let mut sorted = services.clone();
    sorted.sort_unstable();
    assert_eq!(services, sorted, "service rollup is sorted");
    let exercised: Vec<(&str, &str)> = array(&first, "/exercised_operations")?
        .iter()
        .filter_map(|row| Some((row.get("service")?.as_str()?, row.get("module")?.as_str()?)))
        .collect();
    let mut sorted = exercised.clone();
    sorted.sort_unstable();
    assert_eq!(exercised, sorted, "exercised rows are sorted");
    let sample: Vec<&str> = array(&first, "/unexercised_sample")?
        .iter()
        .filter_map(|row| row.get("module").and_then(Value::as_str))
        .collect();
    let mut sorted = sample.clone();
    sorted.sort_unstable();
    assert_eq!(sample, sorted, "unexercised sample is sorted");
    Ok(())
}

#[test]
fn check_detects_document_drift() -> TestResult {
    let directory = fixture()?;
    let root = directory.path();
    run_operation_coverage(root, Some(Path::new("docs/OPERATION_COVERAGE.md")), false)?;
    run_operation_coverage(root, None, true)?;
    let document = root.join("docs/OPERATION_COVERAGE.md");
    let mut text = std::fs::read_to_string(&document)?;
    text.push_str("drift\n");
    std::fs::write(&document, text)?;
    let failure = match run_operation_coverage(root, None, true) {
        Ok(_) => return Err("expected drift failure".into()),
        Err(HarnessError::Invalid(reason)) => reason,
        Err(other) => return Err(other.into()),
    };
    assert!(
        failure.contains("operation coverage document differs"),
        "{failure}"
    );
    assert_eq!(
        match run_operation_coverage(root, Some(document.as_path()), true) {
            Ok(_) => String::from("accepted"),
            Err(error) => error.to_string(),
        },
        "--check and --write are mutually exclusive"
    );
    Ok(())
}

#[test]
fn check_rejects_missing_document() -> TestResult {
    let directory = fixture()?;
    let failure = match run_operation_coverage(directory.path(), None, true) {
        Ok(_) => return Err("expected missing document failure".into()),
        Err(error) => error.to_string(),
    };
    assert!(failure.contains("OPERATION_COVERAGE.md"), "{failure}");
    Ok(())
}
