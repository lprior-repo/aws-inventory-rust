use inventory_harness::HarnessError;
use inventory_harness::audit::{
    Json, apply_verification, checked_histogram, py_eq, request_evidence, validate_summary,
};
use inventory_harness::provenance::canonical_sha256;
use serde_json::{Value, json};
use std::collections::BTreeMap;
type BoxError = Box<dyn std::error::Error>;
type TestResult = Result<(), BoxError>;
fn failure<T>(result: Result<T, HarnessError>) -> Result<String, BoxError> {
    match result {
        Err(error) => Ok(error.to_string()),
        Ok(_) => Err(String::from("expected failure").into()),
    }
}
fn at<'a>(value: &'a Value, path: &str) -> Result<&'a Value, BoxError> {
    value
        .pointer(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}
fn at_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Value, BoxError> {
    value
        .pointer_mut(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}
fn contain(text: &str, needle: &str) -> TestResult {
    assert!(text.contains(needle), "missing {needle:?} in {text:?}");
    Ok(())
}
fn checks(names: &[&str]) -> Value {
    json!(
        names
            .iter()
            .map(|name| json!({"check": name, "passed": true}))
            .collect::<Vec<Value>>()
    )
}
#[test]
fn emulator_summary_must_be_complete_and_identified() -> TestResult {
    let (report, catalog, matrix, summary) = binding_fixture()?;
    assert_eq!(
        validate_summary(&report, &summary, &catalog, &matrix)?
            .1
            .len(),
        1
    );
    let mut missing = summary.clone();
    *at_mut(&mut missing, "/expected_operations")? = json!([]);
    assert_eq!(
        failure(validate_summary(&report, &missing, &catalog, &matrix))?,
        "emulator operation identity set differs from reconciled boundaries"
    );
    let mut relabelled = summary.clone();
    *at_mut(
        &mut relabelled,
        "/expected_operations/0/credential_provider_boundary",
    )? = json!(true);
    contain(
        &failure(validate_summary(&report, &relabelled, &catalog, &matrix))?,
        "provider classification differs",
    )?;
    let mut failed = summary;
    *at_mut(&mut failed, "/checks/0/passed")? = json!(false);
    contain(
        &failure(validate_summary(&report, &failed, &catalog, &matrix))?,
        "complete asserted",
    )
}
#[test]
fn request_ledger_rejects_undeclared_or_overstated_requests() -> TestResult {
    let directory = tempfile::tempdir()?;
    let ledger = directory.path().join("requests.jsonl");
    let records = [
        json!({"service": "example", "operation": "List", "http_status": 200, "phase": "seed:catalog"}),
        json!({"service": "example", "operation": "List", "http_status": 200, "phase": "rust:cloud-control"}),
    ];
    std::fs::write(
        &ledger,
        records
            .iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>(),
    )?;
    let row = json!({"service": "example", "operation": "List", "attempts": 1,
                     "http_statuses": {"200": 1}, "phases": ["rust:cloud-control"]});
    assert_eq!(
        at(
            &request_evidence(&ledger, std::slice::from_ref(&row))?,
            "/records"
        )?,
        &json!(2)
    );
    let overstated = json!({"service": "example", "operation": "List", "attempts": 2,
                            "http_statuses": {"200": 2}, "phases": ["rust:cloud-control"]});
    assert_eq!(
        failure(request_evidence(&ledger, &[overstated]))?,
        "operation summary disagrees with the Rust request ledger"
    );
    Ok(())
}
fn coverage_fixture() -> Result<(Json, Json, Json, Json), BoxError> {
    let catalog = Json::from_value(&json!({"resource_types": [
        {"type_name": "AWS::Example::Thing", "schema_sha256": "a".repeat(64)}]}));
    let matrix = Json::from_value(&json!({
        "resource_types": [{"type_name": "AWS::Example::Thing", "schema_sha256": "a".repeat(64), "selected": true}],
        "summary": {}, "evidence": {},
    }));
    let names = [
        "coverage:all_catalog_types_listed",
        "coverage:every_registered_type_read",
        "sqlite:integrity",
        "cloud_control:architecture_fixture_metadata",
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
    ];
    let summary = Json::from_value(&json!({
        "schema_version": 1, "failed_checks": 0, "passed_checks": names.len(), "checks": checks(&names),
        "cloud_control_resource_types": [{"type_name": "AWS::Example::Thing",
            "operations": {"ListResources": {"200": 1}, "GetResource": {"200": 1}}}],
        "execution": {"catalog_canonical_sha256": canonical_sha256(&catalog.to_value()),
                      "catalog_sha256": "c".repeat(64), "source": {"sha256": "d".repeat(64)},
                      "binary": {"sha256": "e".repeat(64)}, "architecture_fixture_sha256": "f".repeat(64)},
    }));
    let binding = Json::from_value(&json!({"summary_file": "artifacts/example/summary.json",
        "catalog_sha256": "c".repeat(64), "source_sha256": "d".repeat(64),
        "binary_sha256": "e".repeat(64), "architecture_fixture_sha256": "f".repeat(64)}));
    Ok((catalog, matrix, summary, binding))
}
#[test]
fn coverage_requires_current_digests_and_matching_typed_reads() -> TestResult {
    let (catalog, matrix, summary, binding) = coverage_fixture()?;
    let mut counts = BTreeMap::from([("AWS::Example::Thing".to_owned(), 1)]);
    let merged = apply_verification(&matrix, &catalog, &summary, &counts, &binding)?.to_value();
    assert_eq!(
        at(&merged, "/resource_types/0/emulator_verification/status")?,
        &json!("read_observed")
    );
    assert_eq!(at(&merged, "/summary/emulator_read_observed")?, &json!(1));
    let original = matrix.to_value();
    assert_eq!(
        at(&original, "/resource_types/0")?.get("emulator_verification"),
        None
    );
    let stale = Json::from_value(
        &json!({"summary_file": "unbound.json", "catalog_sha256": "c".repeat(64),
        "source_sha256": "d".repeat(64), "binary_sha256": "0".repeat(64), "architecture_fixture_sha256": "f".repeat(64)}),
    );
    assert_eq!(
        failure(apply_verification(
            &matrix, &catalog, &summary, &counts, &stale
        ))?,
        "current catalog/source/binary/fixture differs from executed evidence"
    );
    counts.insert("AWS::Example::Other".to_owned(), 1);
    contain(
        &failure(apply_verification(
            &matrix, &catalog, &summary, &counts, &binding,
        ))?,
        "unexpected provider",
    )?;
    let excessive = BTreeMap::from([("AWS::Example::Thing".to_owned(), 2)]);
    contain(
        &failure(apply_verification(
            &matrix, &catalog, &summary, &excessive, &binding,
        ))?,
        "matching Rust GetResource",
    )?;
    let valid = Json::parse(r#"{"200": 2, "400": 1, "None": 1}"#)?;
    assert!(py_eq(&checked_histogram(&valid)?, &valid));
    for value in [
        r#"{"200": true}"#,
        r#"{"200": 0}"#,
        r#"{"999": 1}"#,
        r#"{"success": 1}"#,
        r#"[]"#,
    ] {
        assert!(
            !failure(checked_histogram(&Json::parse(value)?))?.is_empty(),
            "{value}"
        );
    }
    Ok(())
}

fn binding_fixture() -> Result<(Value, Value, Value, Value), BoxError> {
    let catalog = json!({"resource_types": [{"type_name": "AWS::Example::Thing"}]});
    let matrix = json!({
        "operations": [{"id": "example.list", "boto3_service": "example",
                        "api_operation": "List", "explicit_sdk_send": true}],
        "credential_provider_boundaries": [],
    });
    let report = json!({
        "source": {"sha256": "a".repeat(64)},
        "cloud_control_provider_dimension": {
            "catalog_sha256": "b".repeat(64),
            "parent_catalog_sha256": Value::Null,
        },
    });
    let names = [
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    let summary = json!({
        "schema_version": 1, "failed_checks": 0, "passed_checks": names.len(),
        "checks": checks(&names),
        "execution": {"source": report.get("source").cloned().ok_or_else(|| BoxError::from("source"))?,
                      "catalog_sha256": "b".repeat(64),
                      "catalog_canonical_sha256": canonical_sha256(&catalog)},
        "expected_operations": [{"id": "example.list", "service": "example", "operation": "List",
                                 "credential_provider_boundary": false, "status": "sdk_decode_failure"}],
    });
    Ok((report, catalog, matrix, summary))
}
