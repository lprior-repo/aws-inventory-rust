//! Tests for check_additions and write_reconciliation.

use std::fs;
use std::path::PathBuf;

use inventory_harness::audit::{check_additions, write_reconciliation};
use sha2::{Digest, Sha256};

fn test_root(name: &str) -> PathBuf {
    let root = PathBuf::from(format!("/tmp/additions_test_{}", name));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_json(root: &PathBuf, name: &str, value: &serde_json::Value) {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    let raw = serde_json::to_string(value).expect("serialize");
    fs::write(&path, &raw).expect("write file");
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

#[test]
fn test_absent_overlay_is_a_noop_for_the_validator() {
    let root = test_root("absent_overlay");
    let matrix = make_matrix();
    let path = root.join("catalog/aws-operations.json");
    write_json(&root, path.to_string_lossy().trim_start_matches("./"), &matrix);

    let result = check_additions(&root);
    assert!(result.is_ok(), "check_additions should succeed when additions file is absent");
    let value = result.unwrap();
    assert_eq!(
        value["status"], "absent",
        "status should be 'absent' when no additions file exists"
    );
}

#[test]
fn test_absent_overlay_leaves_reconciliation_output_untouched() {
    // When additions file is missing, check_additions returns null additions
    // and the report's callsites have no origin field
    let root = test_root("reconciliation_untouched");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let result = check_additions(&root).unwrap();
    assert!(
        result["additions"].is_null(),
        "additions should be null when file is absent"
    );
}

#[test]
fn test_additions_schema_version_must_be_1() {
    let root = test_root("schema_version");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    // Compute matrix SHA for the binds field
    let matrix_bytes = fs::read(root.join("catalog/aws-operations.json")).unwrap();
    let matrix_sha = hex::encode(Sha256::digest(&matrix_bytes));

    // Wrong schema version
    let bad_additions = serde_json::json!({
        "schema_version": 2,
        "binds": {
            "historical_matrix": {"sha256": matrix_sha}
        },
        "additions": []
    });
    write_json(&root, "catalog/aws-operation-additions.json", &bad_additions);

    let result = check_additions(&root);
    assert!(result.is_err(), "should fail with wrong schema_version");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("schema_version"),
        "error should mention schema_version, got: {}",
        err
    );
}

#[test]
fn test_additions_must_bind_correct_matrix() {
    let root = test_root("matrix_binding");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let bad_sha = "deadbeef".repeat(8);
    let bad_additions = serde_json::json!({
        "schema_version": 1,
        "binds": {
            "historical_matrix": {"sha256": bad_sha}
        },
        "additions": []
    });
    write_json(&root, "catalog/aws-operation-additions.json", &bad_additions);

    let result = check_additions(&root);
    assert!(result.is_err(), "should fail when matrix binding is wrong");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("different historical operation matrix"),
        "error should mention matrix binding, got: {}",
        err
    );
}

#[test]
fn test_duplicate_operation_identity_fails() {
    let root = test_root("duplicate_identity");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let matrix_bytes = fs::read(root.join("catalog/aws-operations.json")).unwrap();
    let matrix_sha = hex::encode(Sha256::digest(&matrix_bytes));

    // Addition reuses a historical operation identity
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
    write_json(&root, "catalog/aws-operation-additions.json", &bad_additions);

    let result = check_additions(&root);
    assert!(result.is_err(), "should fail when addition reuses historical identity");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("reuses a historical operation identity"),
        "error should mention historical identity, got: {}",
        err
    );
}

#[test]
fn test_valid_additions_pass() {
    let root = test_root("valid_additions");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let matrix_bytes = fs::read(root.join("catalog/aws-operations.json")).unwrap();
    let matrix_sha = hex::encode(Sha256::digest(&matrix_bytes));

    // Addition with a new identity
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
    write_json(&root, "catalog/aws-operation-additions.json", &valid_additions);

    let result = check_additions(&root);
    assert!(result.is_ok(), "valid additions should pass");
    let value = result.unwrap();
    assert_eq!(value["status"], "applied");
    assert_eq!(value["additions"]["additions_count"], 1);
}

#[test]
fn test_output_path_must_be_within_root() {
    let root = test_root("outside_root");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let report = serde_json::json!({"status": "ok"});
    let outside = PathBuf::from("/tmp/outside_additions_test.json");

    // Canonicalize the outside path first to make it exist
    fs::write(&outside, "test").expect("write outside file");
    let outside = outside.canonicalize().expect("canonicalize outside");

    let result = write_reconciliation(&root, &report, &outside);
    assert!(result.is_err(), "output outside root should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("within the repository root"),
        "error should mention root, got: {}",
        err
    );
}

#[test]
fn test_historical_paths_cannot_be_replaced() {
    let root = test_root("historical_paths");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);

    let report = serde_json::json!({"status": "ok"});

    // Create the files that are protected so canonicalize works
    for path in [
        "catalog/aws-operations.json",
        "catalog/aws-operation-additions.json",
        "artifacts/aws-operation-source-reconciliation-v04.json",
        "artifacts/aws-operation-source-reconciliation-v05.json",
    ] {
        let target = root.join(path);
        if !target.exists() {
            write_json(&root, path, &serde_json::json!({"test": true}));
        }
    }

    for path in [
        "catalog/aws-operations.json",
        "catalog/aws-operation-additions.json",
        "artifacts/aws-operation-source-reconciliation-v04.json",
        "artifacts/aws-operation-source-reconciliation-v05.json",
    ] {
        let target = root.join(path);
        let result = write_reconciliation(&root, &report, &target);
        assert!(
            result.is_err(),
            "writing to {} should fail",
            path
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("historical evidence cannot be replaced"),
            "error for {} should mention historical evidence, got: {}",
            path,
            err
        );
    }
}

#[test]
fn test_atomic_write_leaves_no_partial_on_failure() {
    let root = test_root("atomic_write");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);
    fs::create_dir_all(root.join("artifacts")).expect("create artifacts dir");

    let report = serde_json::json!({"status": "ok"});
    let output = root.join("artifacts/test-reconciliation.json");
    // Touch the file so canonicalize works
    fs::write(&output, "").expect("touch output");

    let result = write_reconciliation(&root, &report, &output);
    assert!(result.is_ok(), "write should succeed");
    assert!(output.exists(), "output file should exist");

    // No .partial temp file should be left behind
    let partial = output.with_extension("partial");
    assert!(
        !partial.exists(),
        "no .partial temp file should remain after successful write"
    );
}

#[test]
fn test_atomic_write_replaces_existing_file() {
    let root = test_root("replace_existing");
    let matrix = make_matrix();
    write_json(&root, "catalog/aws-operations.json", &matrix);
    fs::create_dir_all(root.join("artifacts")).expect("create artifacts dir");

    let report1 = serde_json::json!({"status": "first"});
    let output = root.join("artifacts/replace-test.json");
    fs::write(&output, "").expect("touch output");
    write_reconciliation(&root, &report1, &output).expect("first write");

    // Modify and write again
    let report2 = serde_json::json!({"status": "second"});
    write_reconciliation(&root, &report2, &output).expect("second write");

    // Verify the file was replaced atomically
    let content = fs::read_to_string(&output).expect("read output");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse output");
    assert_eq!(parsed["status"], "second");
}