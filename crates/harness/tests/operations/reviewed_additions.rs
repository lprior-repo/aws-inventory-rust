//! Reviewed additive overlay: admission, binding, no-success and no-op contracts.
use super::*;
use inventory_harness::provenance::{file_sha256, sha256_hex};

const OVERLAY: &str = "catalog/aws-operation-additions.json";

pub(super) fn overlay_fixture() -> Result<tempfile::TempDir, BoxError> {
    let directory = fixture()?;
    install_overlay(directory.path())?;
    Ok(directory)
}

fn install_overlay(root: &Path) -> Result<(), BoxError> {
    let inventory_sha = add_kms_inventory(root)?;
    let source = "use aws_sdk_kms::Client;\nfn list_keys(client: &Client) {\n    \
                    client.list_keys().send();\n}\n";
    let source_sha = sha256_hex(source.as_bytes());
    std::fs::write(root.join("crates/aws/src/kms.rs"), source)?;
    let matrix_sha = file_sha256(&root.join("catalog/aws-operations.json"))?;
    let overlay = overlay_document(&matrix_sha, &inventory_sha, &source_sha);
    std::fs::write(root.join(OVERLAY), serde_json::to_vec_pretty(&overlay)?)?;
    Ok(())
}

fn add_kms_inventory(root: &Path) -> Result<String, BoxError> {
    let path = root.join("catalog/aws-sdk-operations.json");
    let mut inventory: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let crates = inventory
        .get_mut("crates")
        .and_then(Value::as_array_mut)
        .ok_or("crates")?;
    crates.push(
        json!({"name": "aws-sdk-kms", "version": "1.0.0", "operations": [
            {"module": "list_keys", "api_operation": "ListKeys"},
        ]}),
    );
    let summary = inventory
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .ok_or("summary")?;
    summary.insert("crates".to_owned(), json!(4));
    summary.insert("operations".to_owned(), json!(8));
    let bytes = serde_json::to_vec_pretty(&inventory)?;
    std::fs::write(&path, &bytes)?;
    Ok(sha256_hex(&bytes))
}

fn overlay_document(matrix_sha: &str, inventory_sha: &str, source_sha: &str) -> Value {
    let model_sha = "a".repeat(64);
    let module_sha = "b".repeat(64);
    json!({
        "schema_version": 1,
        "scope": "fixture reviewed additions",
        "binds": {
            "historical_matrix": {"path": "catalog/aws-operations.json", "sha256": matrix_sha},
            "operation_inventory": {"path": "catalog/aws-sdk-operations.json", "sha256": inventory_sha},
        },
        "review": {"reviewed_at_utc": "2026-11-13T00:00:00Z", "method": "fixture"},
        "additions": [{
            "id": "kms.list_keys", "sdk_service": "kms", "boto3_service": "kms",
            "sdk_crate": "aws-sdk-kms", "sdk_version": "1.0.0",
            "sdk_operation": "list_keys", "api_operation": "ListKeys",
            "category": "native_regional", "explicit_sdk_send": true,
            "source_calls": [{"path": "crates/aws/src/kms.rs", "function": "list_keys",
                              "builder_line": 3, "operation_line": 3, "send_line": 3}],
            "source_sha256": {"crates/aws/src/kms.rs": source_sha},
            "wire_source": {"kind": "fixture", "service": "kms", "api_version": "2014-11-01",
                            "operation_model_sha256": model_sha,
                            "rust_operation_module": "src/operation/list_keys.rs",
                            "rust_operation_module_sha256": module_sha},
            "coverage": {"status": "pending_execution", "reason": "fixture review",
                         "execution_receipt": null},
        }],
    })
}

fn set(root: &Path, path: &str, value: Value) -> Result<(), BoxError> {
    let file = root.join(OVERLAY);
    let mut document: Value = serde_json::from_slice(&std::fs::read(&file)?)?;
    let slot = document
        .pointer_mut(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))?;
    *slot = value;
    std::fs::write(&file, serde_json::to_vec_pretty(&document)?)?;
    Ok(())
}

fn coverage_failure(root: &Path) -> Result<String, BoxError> {
    match operation_coverage(root) {
        Ok(_) => Err(BoxError::from("expected failure")),
        Err(error) => Ok(error.to_string()),
    }
}

#[test]
fn reviewed_additions_are_admitted_without_becoming_exercised() -> TestResult {
    let directory = overlay_fixture()?;
    let report = operation_coverage(directory.path())?.to_value()?;
    assert_eq!(number(&report, "/summary/operations_total")?, 8);
    assert_eq!(number(&report, "/summary/exercised")?, 4);
    assert_eq!(number(&report, "/summary/unexercised")?, 4);
    assert_eq!(number(&report, "/summary/reviewed_additions")?, 1);
    assert_eq!(number(&report, "/summary/reviewed_additions_exercised")?, 0);
    assert_eq!(
        pointer(&report, "/reviewed_additions/0/execution_status")?,
        &json!("unexercised")
    );
    assert_eq!(
        pointer(&report, "/reviewed_additions/0/status")?,
        &json!("pending_execution")
    );
    assert_eq!(
        pointer(&report, "/sources/operation_additions/status")?,
        &json!("applied")
    );
    let digest = pointer(&report, "/sources/operation_additions/sha256")?
        .as_str()
        .ok_or("missing overlay digest")?;
    assert_eq!(digest.len(), 64);
    assert!(
        array(&report, "/exercised_operations")?
            .iter()
            .all(|row| { row.get("service").and_then(Value::as_str) != Some("kms") })
    );
    assert!(
        array(&report, "/unexercised_sample")?
            .iter()
            .all(|row| { row.get("module").and_then(Value::as_str) != Some("list_keys") })
    );
    Ok(())
}

#[test]
fn addition_reusing_a_matrix_operation_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    let root = directory.path();
    set(root, "/additions/0/id", json!("alpha.list_things"))?;
    set(root, "/additions/0/sdk_service", json!("alpha"))?;
    set(root, "/additions/0/boto3_service", json!("alpha"))?;
    set(root, "/additions/0/sdk_operation", json!("list_things"))?;
    set(root, "/additions/0/wire_source/service", json!("alpha"))?;
    set(
        root,
        "/additions/0/wire_source/rust_operation_module",
        json!("src/operation/list_things.rs"),
    )?;
    let failure = coverage_failure(root)?;
    assert!(
        failure.contains("reuses a historical operation identity"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn addition_with_stale_matrix_binding_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    set(
        directory.path(),
        "/binds/historical_matrix/sha256",
        json!("0".repeat(64)),
    )?;
    let failure = coverage_failure(directory.path())?;
    assert!(
        failure.contains("bind a different historical operation matrix"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn addition_whose_callsite_function_moved_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    set(
        directory.path(),
        "/additions/0/source_calls/0/function",
        json!("moved"),
    )?;
    let failure = coverage_failure(directory.path())?;
    assert!(
        failure.contains("callsites changed; re-review"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn addition_claiming_emulator_success_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    set(
        directory.path(),
        "/additions/0/coverage/status",
        json!("collector_success"),
    )?;
    let failure = coverage_failure(directory.path())?;
    assert!(
        failure.contains("must not claim an emulator outcome"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn addition_with_changed_reviewed_source_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    set(
        directory.path(),
        "/additions/0/source_sha256/crates~1aws~1src~1kms.rs",
        json!("0".repeat(64)),
    )?;
    let failure = coverage_failure(directory.path())?;
    assert!(
        failure.contains("source changed since review: crates/aws/src/kms.rs"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn addition_naming_an_unpinned_api_operation_is_rejected() -> TestResult {
    let directory = overlay_fixture()?;
    let root = directory.path();
    set(root, "/additions/0/id", json!("kms.no_such"))?;
    set(root, "/additions/0/sdk_operation", json!("no_such"))?;
    set(
        root,
        "/additions/0/wire_source/rust_operation_module",
        json!("src/operation/no_such.rs"),
    )?;
    let failure = coverage_failure(root)?;
    assert!(
        failure.contains("is absent from catalog/aws-sdk-operations.json"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn document_renders_additions_and_check_roundtrips() -> TestResult {
    let directory = overlay_fixture()?;
    let root = directory.path();
    run_operation_coverage(root, Some(Path::new("docs/OPERATION_COVERAGE.md")), false)?;
    run_operation_coverage(root, None, true)?;
    let document = std::fs::read_to_string(root.join("docs/OPERATION_COVERAGE.md"))?;
    assert!(
        document.contains("## Reviewed additive operations (not exercised)"),
        "{document}"
    );
    set(root, "/additions/0/source_calls/0/function", json!("moved"))?;
    let failure = match run_operation_coverage(root, None, true) {
        Ok(_) => return Err(BoxError::from("expected overlay drift failure")),
        Err(error) => error.to_string(),
    };
    assert!(
        failure.contains("callsites changed; re-review"),
        "{failure}"
    );
    Ok(())
}

#[test]
fn absent_overlay_adds_nothing_to_document_or_json() -> TestResult {
    let directory = fixture()?;
    let root = directory.path();
    let report = operation_coverage(root)?.to_value()?;
    assert_eq!(
        pointer(&report, "/sources/operation_additions")?,
        &Value::Null
    );
    assert_eq!(number(&report, "/summary/reviewed_additions")?, 0);
    assert_eq!(number(&report, "/summary/reviewed_additions_exercised")?, 0);
    assert!(array(&report, "/reviewed_additions")?.is_empty());
    run_operation_coverage(root, Some(Path::new("docs/OPERATION_COVERAGE.md")), false)?;
    run_operation_coverage(root, None, true)?;
    let document = std::fs::read_to_string(root.join("docs/OPERATION_COVERAGE.md"))?;
    assert!(!document.contains("Reviewed additive"), "{document}");
    assert!(!document.contains(OVERLAY), "{document}");
    Ok(())
}

#[test]
fn coverage_write_cannot_replace_the_overlay() -> TestResult {
    let directory = overlay_fixture()?;
    let root = directory.path();
    let before = std::fs::read(root.join(OVERLAY))?;
    let failure = match run_operation_coverage(root, Some(Path::new(OVERLAY)), false) {
        Ok(_) => return Err(BoxError::from("expected protected overlay failure")),
        Err(error) => error.to_string(),
    };
    assert!(failure.contains("cannot be replaced"), "{failure}");
    assert_eq!(std::fs::read(root.join(OVERLAY))?, before);
    Ok(())
}
