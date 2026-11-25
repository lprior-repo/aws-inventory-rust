use std::fs;
use std::path::{Path, PathBuf};

use inventory_harness::audit::{check_additions, write_reconciliation};
use sha2::{Digest, Sha256};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn test_root(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = PathBuf::from(format!("/tmp/additions_test_{name}"));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_json(root: &Path, name: &str, value: &serde_json::Value) -> TestResult {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string(value)?;
    fs::write(&path, &raw)?;
    Ok(())
}

fn write_matrix(root: &Path, value: &serde_json::Value) -> TestResult {
    let path = root.join("catalog/aws-operations.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, inventory_harness::catalog::toml_doc::render(value)?)?;
    Ok(())
}

fn make_matrix() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "operations": [
            {
                "id": "alpha.list_things",
                "sdk_crate": "aws-sdk-alpha",
                "sdk_operation": "list_things",
                "source_calls": []
            }
        ],
        "extraction": {"source_sha256": {}},
        "credential_provider_boundaries": [],
        "cloud_control_provider_dimension": {"resource_type_count": 1}
    })
}

fn matrix_sha(root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = root.join("catalog/aws-operations.toml");
    let matrix_bytes = fs::read(path)?;
    let digest = Sha256::digest(&matrix_bytes);
    Ok(hex::encode(digest))
}

#[test]
fn test_absent_overlay_is_a_noop_for_the_validator() -> TestResult {
    let root = test_root("absent_overlay")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let value = check_additions(&root)?;
    assert_eq!(
        value.get("status").and_then(|v| v.as_str()),
        Some("absent"),
        "status should be 'absent' when no additions file exists"
    );
    Ok(())
}

#[test]
fn test_absent_overlay_leaves_reconciliation_output_untouched() -> TestResult {
    let root = test_root("reconciliation_untouched")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let result = check_additions(&root)?;
    assert!(
        result
            .get("additions")
            .is_some_and(serde_json::Value::is_null),
        "additions should be null when file is absent"
    );
    Ok(())
}

#[test]
fn test_additions_schema_version_must_be_1() -> TestResult {
    let root = test_root("schema_version")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let matrix_sha = matrix_sha(&root)?;

    let bad_additions = serde_json::json!({
        "schema_version": 2,
        "binds": {
            "historical_matrix": {"sha256": matrix_sha}
        },
        "additions": []
    });
    write_json(
        &root,
        "catalog/aws-operation-additions.json",
        &bad_additions,
    )?;

    let result = check_additions(&root);
    let Err(e) = result else {
        return Err("expected check_additions to fail with wrong schema_version".into());
    };
    let err = e.to_string();
    assert!(
        err.contains("schema_version"),
        "error should mention schema_version, got: {err}"
    );
    Ok(())
}

#[test]
fn test_additions_must_bind_correct_matrix() -> TestResult {
    let root = test_root("matrix_binding")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let bad_sha = "deadbeef".repeat(8);
    let bad_additions = serde_json::json!({
        "schema_version": 1,
        "binds": {
            "historical_matrix": {"sha256": bad_sha}
        },
        "additions": []
    });
    write_json(
        &root,
        "catalog/aws-operation-additions.json",
        &bad_additions,
    )?;

    let result = check_additions(&root);
    let Err(e) = result else {
        return Err("expected check_additions to fail with wrong matrix binding".into());
    };
    let err = e.to_string();
    assert!(
        err.contains("different historical operation matrix"),
        "error should mention matrix binding, got: {err}"
    );
    Ok(())
}

#[test]
fn test_duplicate_operation_identity_fails() -> TestResult {
    let root = test_root("duplicate_identity")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let matrix_sha = matrix_sha(&root)?;

    let bad_additions = serde_json::json!({
        "schema_version": 1,
        "binds": {
            "historical_matrix": {"sha256": matrix_sha}
        },
        "additions": [{
            "id": "alpha.list_things",
            "sdk_service": "alpha",
            "sdk_crate": "aws-sdk-alpha",
            "sdk_version": "1.0.0",
            "sdk_operation": "list_things",
            "api_operation": "ListThings",
            "explicit_sdk_send": true
        }]
    });
    write_json(
        &root,
        "catalog/aws-operation-additions.json",
        &bad_additions,
    )?;

    let result = check_additions(&root);
    let Err(e) = result else {
        return Err("expected check_additions to fail with duplicate identity".into());
    };
    let err = e.to_string();
    assert!(
        err.contains("reuses a historical operation identity"),
        "error should mention historical identity, got: {err}"
    );
    Ok(())
}

#[test]
fn test_valid_additions_pass() -> TestResult {
    let root = test_root("valid_additions")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let matrix_sha = matrix_sha(&root)?;

    let valid_additions = serde_json::json!({
        "schema_version": 1,
        "binds": {
            "historical_matrix": {"sha256": matrix_sha}
        },
        "additions": [{
            "id": "beta.new_thing",
            "sdk_service": "beta",
            "sdk_crate": "aws-sdk-beta",
            "sdk_version": "1.0.0",
            "sdk_operation": "new_thing",
            "api_operation": "NewThing",
            "explicit_sdk_send": true
        }]
    });
    write_json(
        &root,
        "catalog/aws-operation-additions.json",
        &valid_additions,
    )?;

    let value = check_additions(&root)?;
    assert_eq!(
        value.get("status").and_then(|v| v.as_str()),
        Some("applied")
    );
    assert_eq!(
        value
            .get("additions")
            .and_then(|a| a.get("additions_count"))
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    Ok(())
}

#[test]
fn test_output_path_must_be_within_root() -> TestResult {
    let root = test_root("outside_root")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let report = serde_json::json!({"status": "ok"});
    let outside = PathBuf::from("/tmp/outside_additions_test.json");
    fs::write(&outside, "test")?;
    let outside = outside.canonicalize()?;

    let result = write_reconciliation(&root, &report, &outside);
    let Err(e) = result else {
        return Err("expected write_reconciliation to fail for output outside root".into());
    };
    let err = e.to_string();
    assert!(
        err.contains("within the repository root"),
        "error should mention root, got: {err}"
    );
    Ok(())
}

#[test]
fn test_historical_paths_cannot_be_replaced() -> TestResult {
    let root = test_root("historical_paths")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;

    let report = serde_json::json!({"status": "ok"});

    let protected_paths = [
        "catalog/aws-operations.toml",
        "catalog/aws-operation-additions.json",
        "artifacts/aws-operation-source-reconciliation-v04.json",
        "artifacts/aws-operation-source-reconciliation-v05.json",
    ];
    for path in &protected_paths {
        let target = root.join(path);
        if !target.exists() {
            write_json(&root, path, &serde_json::json!({"test": true}))?;
        }
    }

    for path in &protected_paths {
        let target = root.join(path);
        let result = write_reconciliation(&root, &report, &target);
        let Err(e) = result else {
            return Err(format!("expected write to {path} to fail").into());
        };
        let err = e.to_string();
        assert!(
            err.contains("historical evidence cannot be replaced"),
            "error for {path} should mention historical evidence, got: {err}"
        );
    }
    Ok(())
}

#[test]
fn test_atomic_write_leaves_no_partial_on_failure() -> TestResult {
    let root = test_root("atomic_write")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;
    fs::create_dir_all(root.join("artifacts"))?;

    let report = serde_json::json!({"status": "ok"});
    let output = root.join("artifacts/test-reconciliation.json");
    fs::write(&output, "")?;

    write_reconciliation(&root, &report, &output)?;
    assert!(output.exists(), "output file should exist");

    let partial = output.with_extension("partial");
    assert!(
        !partial.exists(),
        "no .partial temp file should remain after successful write"
    );
    Ok(())
}

#[test]
fn test_atomic_write_replaces_existing_file() -> TestResult {
    let root = test_root("replace_existing")?;
    let matrix = make_matrix();
    write_matrix(&root, &matrix)?;
    fs::create_dir_all(root.join("artifacts"))?;

    let report1 = serde_json::json!({"status": "first"});
    let output = root.join("artifacts/replace-test.json");
    fs::write(&output, "")?;
    write_reconciliation(&root, &report1, &output)?;

    let report2 = serde_json::json!({"status": "second"});
    write_reconciliation(&root, &report2, &output)?;

    let content = fs::read_to_string(&output)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(
        parsed.get("status").and_then(|v| v.as_str()),
        Some("second")
    );
    Ok(())
}
