use std::fs;
use std::path::PathBuf;

use hex;

use inventory_harness::audit::canonical_sha256;

use inventory_harness::audit::validate_summary;
use sha2::Digest;

fn test_root(name: &str) -> PathBuf {
    let root = PathBuf::from(format!("/tmp/audit_test_{}", name));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove test root");
    }
    fs::create_dir_all(&root).expect("create test root");
    root
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn write_json(root: &PathBuf, name: &str, value: &serde_json::Value) -> String {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    let raw = serde_json::to_string(value).expect("serialize");
    fs::write(&path, &raw).expect("write file");
    sha256_hex(raw.as_bytes())
}

fn make_report() -> serde_json::Value {
    serde_json::json!({
        "source": {"sha256": "a".repeat(64), "files": []},
        "cloud_control_provider_dimension": {"catalog_sha256": "b".repeat(64)}
    })
}

fn make_matrix() -> serde_json::Value {
    serde_json::json!({
        "operations": [{"id": "example.list", "boto3_service": "example",
            "api_operation": "List", "explicit_sdk_send": true}],
        "credential_provider_boundaries": []
    })
}

fn make_catalog() -> serde_json::Value {
    serde_json::json!({
        "resource_types": [{"type_name": "AWS::Example::Thing", "list_requires_model": false}]
    })
}

fn setup_catalogs(root: &PathBuf) -> (serde_json::Value, String) {
    let matrix = make_matrix();
    let matrix_sha = write_json(root, "catalog/aws-operations.json", &matrix);

    let catalog = make_catalog();
    let catalog_sha = write_json(root, "catalog/cloudcontrol-resources.json", &catalog);

    let parent = serde_json::json!({
        "schema_version": 1,
        "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": true}],
        "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster",
            "bindings": [{"source_pointer": "/Name", "model_property": "ClusterName"}]}],
        "evidence": {"root_catalog_sha256": catalog_sha}
    });
    write_json(root, "catalog/cloudcontrol-parent-resources.json", &parent);

    (catalog, matrix_sha)
}

fn make_summary(
    source_sha: &str,
    catalog_sha: &str,
    catalog_canonical_sha: &str,
    status: &str,
) -> serde_json::Value {
    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": source_sha, "files": []},
            "catalog_sha256": catalog_sha,
            "catalog_canonical_sha256": catalog_canonical_sha
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": status}
        ]
    })
}

#[test]
fn test_decoding_failure_remains_distinct_from_success() {
    let report = make_report();
    let matrix = make_matrix();
    let catalog = make_catalog();

    let canonical = canonical_sha256(&catalog);
    let summary = make_summary("a".repeat(64).as_str(), "b".repeat(64).as_str(), &canonical, "sdk_decode_failure");

    let (executed, rows) = validate_summary(&report, &summary, &catalog, &matrix).expect("validate_summary should succeed");
    assert_eq!(rows[0]["status"], "sdk_decode_failure", "decode failure status must be preserved");
    assert_eq!(executed["source"]["sha256"], "a".repeat(64));
}

#[test]
fn test_changed_source_or_descriptor_cannot_reuse_an_old_run() {
    let report = make_report();
    let matrix = make_matrix();
    let catalog = make_catalog();
    let canonical = canonical_sha256(&catalog);

    // Changed source
    let summary = make_summary("c".repeat(64).as_str(), "b".repeat(64).as_str(), &canonical, "sdk_decode_failure");
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("different source or catalog"), "expected source mismatch: {}", err);

    // Changed catalog
    let mut changed_catalog = catalog.clone();
    changed_catalog["resource_types"][0]["relationships"] = serde_json::json!([{"relation": "CHANGED"}]);
    let canonical_changed = canonical_sha256(&changed_catalog);
    let summary = make_summary("a".repeat(64).as_str(), "b".repeat(64).as_str(), &canonical_changed, "sdk_decode_failure");
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("different source or catalog"), "expected catalog mismatch: {}", err);
}

#[test]
fn test_provider_boundary_cannot_be_relabelled_as_an_explicit_sdk_send() {
    let report = make_report();
    let matrix = make_matrix();
    let catalog = make_catalog();
    let canonical = canonical_sha256(&catalog);

    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": true, "status": "sdk_decode_failure"}
        ]
    });

    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("provider classification"), "expected provider classification error: {}", err);
}

#[test]
fn test_duplicate_or_missing_operation_identity_fails_closed() {
    let report = make_report();
    let catalog = make_catalog();
    let canonical = canonical_sha256(&catalog);

    // Duplicate operation
    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"},
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });

    let matrix = make_matrix();
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("identity set"), "expected identity set error: {}", err);
}

#[test]
fn test_failed_or_missing_provenance_checks_cannot_bind_execution() {
    let report = make_report();
    let matrix = make_matrix();
    let catalog = make_catalog();
    let canonical = canonical_sha256(&catalog);

    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 1,
        "passed_checks": 5,
        "checks": [
            {"check": "provenance:embedded_catalog_matches", "passed": false},
            {"check": "provenance:source_unchanged", "passed": true},
            {"check": "provenance:binary_unchanged", "passed": true},
            {"check": "provenance:fixture_unchanged", "passed": true},
            {"check": "coverage:every_operation_attempted", "passed": true},
            {"check": "coverage:no_unexplained_operation_failures", "passed": true}
        ],
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });

    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("complete asserted"), "expected complete asserted error: {}", err);
}

#[test]
fn test_parent_and_merged_catalog_digests_are_required_for_new_evidence() {
    let mut report = make_report();
    let dimension = report["cloud_control_provider_dimension"].as_object_mut().expect("dimension");
    dimension.insert("parent_catalog_sha256".to_string(), serde_json::json!("c".repeat(64)));
    dimension.insert("runtime_catalog_canonical_sha256".to_string(), serde_json::json!("d".repeat(64)));

    let matrix = make_matrix();
    let catalog = make_catalog();
    let canonical = canonical_sha256(&catalog);

    // Missing both
    let summary = make_summary("a".repeat(64).as_str(), "b".repeat(64).as_str(), &canonical, "sdk_decode_failure");
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("different parent or merged"), "expected parent/merged error: {}", err);

    // Wrong parent
    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures",
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical,
            "parent_catalog_sha256": "e".repeat(64),
            "runtime_catalog_canonical_sha256": "d".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("different parent or merged"), "expected parent mismatch: {}", err);

    // Wrong merged
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical,
            "parent_catalog_sha256": "c".repeat(64),
            "runtime_catalog_canonical_sha256": "e".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });
    let err = validate_summary(&report, &summary, &catalog, &matrix).unwrap_err().to_string();
    assert!(err.contains("different parent or merged"), "expected merged mismatch: {}", err);

    // Correct
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64),
            "catalog_canonical_sha256": canonical,
            "parent_catalog_sha256": "c".repeat(64),
            "runtime_catalog_canonical_sha256": "d".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });
    let (executed, rows) = validate_summary(&report, &summary, &catalog, &matrix).expect("should succeed with correct digests");
    assert_eq!(rows[0]["status"], "sdk_decode_failure");
}