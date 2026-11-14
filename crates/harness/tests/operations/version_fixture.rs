//! Mixed-version SDK inventory and source-bound receipt fixtures.
use super::*;
use inventory_harness::provenance::sha256_hex;

pub(super) const INVENTORY: &str = "catalog/aws-sdk-operations.json";
pub(super) const MATRIX: &str = "catalog/aws-operations.json";
const RECEIPT: &str = "artifacts/aws-operation-source-reconciliation-v06.json";

pub(super) fn read(root: &Path, path: &str) -> Result<Value, BoxError> {
    Ok(serde_json::from_slice(&std::fs::read(root.join(path))?)?)
}

pub(super) fn write(root: &Path, path: &str, value: &Value) -> TestResult {
    let bytes = serde_json::to_vec_pretty(value)?;
    Ok(std::fs::write(root.join(path), bytes)?)
}

fn module_digest(version: &str) -> String {
    if version == "1.0.0" {
        "a".repeat(64)
    } else {
        "b".repeat(64)
    }
}

fn alpha(version: &str) -> Value {
    json!({"name":"aws-sdk-alpha", "version":version, "operations":[
        {"module":"list_things", "api_operation":"ListThings", "source_sha256":module_digest(version)},
        {"module":"alpha_only", "api_operation":"AlphaOnly", "source_sha256":module_digest(version)}
    ]})
}

fn inventory(root: &Path) -> TestResult {
    let mut document = read(root, INVENTORY)?;
    let crates = document
        .get_mut("crates")
        .and_then(Value::as_array_mut)
        .ok_or("crates")?;
    crates.retain(|row| row.get("name") != Some(&json!("aws-sdk-alpha")));
    crates.extend([alpha("1.0.0"), alpha("2.0.0")]);
    let summary = document
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .ok_or("summary")?;
    summary.insert("crates".into(), json!(4));
    summary.insert("operations".into(), json!(9));
    write(root, INVENTORY, &document)
}

fn matrix(root: &Path) -> TestResult {
    let mut document = read(root, MATRIX)?;
    document
        .get_mut("operations")
        .and_then(Value::as_array_mut)
        .ok_or("matrix operations")?
        .iter_mut()
        .filter(|row| row.get("sdk_crate") == Some(&json!("aws-sdk-alpha")))
        .try_for_each(|row| -> TestResult {
            row.as_object_mut()
                .ok_or("matrix row")?
                .insert("sdk_version".into(), json!("1.0.0"));
            Ok(())
        })?;
    write(root, MATRIX, &document)
}

fn pin(root: &Path, version: &str) -> TestResult {
    let lock = format!(
        "[[package]]\nname = \"aws-sdk-alpha\"\nversion = \"{version}\"\n\
        [[package]]\nname = \"aws-sdk-beta\"\nversion = \"\"\n\
        [[package]]\nname = \"aws-sdk-cloudcontrol\"\nversion = \"\"\n"
    );
    Ok(std::fs::write(root.join("Cargo.lock"), lock)?)
}

pub(super) fn versioned_fixture(active: &str) -> Result<tempfile::TempDir, BoxError> {
    let directory = fixture()?;
    inventory(directory.path())?;
    matrix(directory.path())?;
    pin(directory.path(), active)?;
    Ok(directory)
}

pub(super) fn receipt(root: &Path, version: &str) -> Result<Value, BoxError> {
    let source = std::fs::read(root.join("crates/aws/src/collect.rs"))?;
    Ok(json!({
        "aws_source_files": [{"path":"crates/aws/src/collect.rs", "sha256":sha256_hex(&source)}],
        "pinned_sdk_operation_modules": [{"id":"alpha.list_things", "sdk_crate":"aws-sdk-alpha",
            "sdk_version":version, "api_operation":"ListThings", "module_sha256":module_digest(version)}],
        "emulator_evidence": {"operation_results":[{"id":"alpha.list_things", "status":"collector_success"}]}
    }))
}

pub(super) fn install_receipt(root: &Path, document: &Value) -> TestResult {
    std::fs::create_dir_all(root.join("artifacts"))?;
    write(root, RECEIPT, document)
}

pub(super) fn set(document: &mut Value, path: &str, value: Value) -> TestResult {
    let slot = document
        .pointer_mut(path)
        .ok_or_else(|| format!("missing {path}"))?;
    *slot = value;
    Ok(())
}

pub(super) fn failure(root: &Path) -> Result<String, BoxError> {
    match operation_coverage(root) {
        Err(HarnessError::Invalid(reason)) => Ok(reason),
        Err(other) => Err(other.into()),
        Ok(_) => Err("expected invalid provenance".into()),
    }
}

pub(super) fn remove_old_inventory(root: &Path) -> TestResult {
    let mut document = read(root, INVENTORY)?;
    document
        .get_mut("crates")
        .and_then(Value::as_array_mut)
        .ok_or("crates")?
        .retain(|row| row.get("version") != Some(&json!("1.0.0")));
    let summary = document
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .ok_or("summary")?;
    summary.insert("crates".into(), json!(3));
    summary.insert("operations".into(), json!(7));
    write(root, INVENTORY, &document)
}

pub(super) fn assert_scan_preserves_lexical_calls(report: &Value) -> TestResult {
    assert_eq!(
        pointer(report, "/sources/callsite_scan/status")?,
        &json!("verified")
    );
    assert_eq!(number(report, "/sources/callsite_scan/callsites")?, 6);
    Ok(())
}
