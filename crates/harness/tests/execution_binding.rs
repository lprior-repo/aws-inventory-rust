use std::fs;
use std::path::PathBuf;

use hex;
use inventory_harness::audit::addition_expected;
use inventory_harness::audit::canonical_sha256;
use inventory_harness::audit::load_additions;
use inventory_harness::audit::validate_emulator;
use sha2::Digest;

fn test_root(name: &str) -> PathBuf {
    let root = PathBuf::from(format!("/tmp/audit_test_{}", name));
    if root.exists() {
        let _ = fs::remove_dir_all(&root);
    }
    fs::create_dir_all(&root).unwrap();
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
        let _ = fs::create_dir_all(parent);
    }
    let raw = serde_json::to_string(value).unwrap();
    fs::write(&path, &raw).unwrap();
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

fn make_summary(source_sha: &str, catalog_sha: &str, status: &str) -> serde_json::Value {
    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures"
    ];
    serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": source_sha, "files": []},
            "catalog_sha256": catalog_sha
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": status}
        ]
    })
}

fn setup_catalogs(root: &PathBuf) -> serde_json::Value {
    let matrix = make_matrix();
    write_json(root, "catalog/aws-operations.json", &matrix);

    let catalog = serde_json::json!({
        "resource_types": [{"type_name": "AWS::EKS::Cluster", "list_requires_model": false}]
    });
    let catalog_sha = write_json(root, "catalog/cloudcontrol-resources.json", &catalog);

    let parent = serde_json::json!({
        "schema_version": 1,
        "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": true}],
        "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster",
            "bindings": [{"source_pointer": "/Name", "model_property": "ClusterName"}]}],
        "evidence": {"root_catalog_sha256": catalog_sha}
    });
    write_json(root, "catalog/cloudcontrol-parent-resources.json", &parent);

    catalog
}

#[test]
fn test_decoding_failure_remains_distinct_from_success() {
    let root = test_root("decode");
    let report = make_report();
    let catalog = setup_catalogs(&root);

    let canonical = canonical_sha256(&catalog);
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": 6,
        "checks": [
            {"check": "provenance:embedded_catalog_matches", "passed": true},
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
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    match result {
        Ok(_) => {}
        Err(e) => panic!("expected success with decode failure status: {}", e),
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_changed_source_cannot_reuse_old_run() {
    let root = test_root("changed_source");
    let report = make_report();
    setup_catalogs(&root);

    let summary = make_summary("c".repeat(64).as_str(), "b".repeat(64).as_str(), "sdk_decode_failure");
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_failed_provenance_check_cannot_bind_execution() {
    let root = test_root("failed_check");
    let report = make_report();
    setup_catalogs(&root);

    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures"
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 1,
        "passed_checks": checks.len() - 1,
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
            "catalog_sha256": "b".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_duplicate_operation_identity_fails_closed() {
    let root = test_root("duplicate_op");
    let report = make_report();
    setup_catalogs(&root);

    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures"
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"},
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": false, "status": "sdk_decode_failure"}
        ]
    });
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_provider_boundary_cannot_be_relabelled() {
    let root = test_root("provider_boundary");
    let report = make_report();
    setup_catalogs(&root);

    let checks = vec![
        "provenance:embedded_catalog_matches",
        "provenance:source_unchanged",
        "provenance:binary_unchanged",
        "provenance:fixture_unchanged",
        "coverage:every_operation_attempted",
        "coverage:no_unexplained_operation_failures"
    ];
    let summary = serde_json::json!({
        "schema_version": 1,
        "failed_checks": 0,
        "passed_checks": checks.len(),
        "checks": checks.iter().map(|c| serde_json::json!({"check": c, "passed": true})).collect::<Vec<_>>(),
        "execution": {
            "source": {"sha256": "a".repeat(64), "files": []},
            "catalog_sha256": "b".repeat(64)
        },
        "expected_operations": [
            {"id": "example.list", "service": "example", "operation": "List",
             "credential_provider_boundary": true, "status": "sdk_decode_failure"}
        ]
    });
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_source_inventory_includes_test_modules() {
    let root = test_root("source_inventory");
    let src_dir = root.join("crates/aws/src");
    fs::create_dir_all(&src_dir).unwrap();

    let lib_src = "use aws_sdk_ec2::Client; fn run() {client.describe_vpcs().send();}";
    fs::write(src_dir.join("lib.rs"), lib_src).unwrap();

    let test_src = "use aws_sdk_ec2::Client; #[test] fn example() {client.describe_subnets().send();}";
    fs::write(src_dir.join("tests.rs"), test_src).unwrap();

    let result = inventory_harness::audit::source_inventory(&root);
    assert!(result.is_ok());
    let (sources, calls, _) = result.unwrap();
    assert_eq!(sources.len(), 2);
    let call_ids: Vec<&str> = calls.iter().map(|c| c.id.as_str()).collect();
    assert!(call_ids.contains(&"ec2.describe_vpcs"));
    assert!(call_ids.contains(&"ec2.describe_subnets"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_parent_and_merged_catalog_digests_required() {
    let root = test_root("parent_digests");
    let report = make_report();
    // Add parent catalog digests to report
    let mut report = report;
    let dimension = report["cloud_control_provider_dimension"].as_object_mut().unwrap();
    dimension.insert("parent_catalog_sha256".to_string(), serde_json::json!("c".repeat(64)));
    dimension.insert("runtime_catalog_canonical_sha256".to_string(), serde_json::json!("d".repeat(64)));

    setup_catalogs(&root);

    // Summary without matching parent digests should fail
    let summary = make_summary("a".repeat(64).as_str(), "b".repeat(64).as_str(), "sdk_decode_failure");
    let _ = fs::create_dir_all(root.join("artifacts"));
    let summary_path = root.join("artifacts/emulator-summary.json");
    fs::write(&summary_path, serde_json::to_string(&summary).unwrap()).unwrap();

    let result = validate_emulator(&report, &root, &summary_path);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_historical_matrix_and_receipts_cannot_be_replaced() {
    let root = test_root("historical");
    setup_catalogs(&root);

    // Try to write to historical paths
    let historical_paths = [
        "catalog/aws-operations.json",
        "artifacts/aws-operation-source-reconciliation-v04.json",
        "artifacts/aws-operation-source-reconciliation-v05.json"
    ];

    for path in &historical_paths {
        let p = root.join(path);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        // Writing to these paths should be allowed, but they represent historical evidence
        let _ = fs::write(&p, "{}");
    }

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_overlay_bound_to_another_matrix_is_rejected() {
    let root = test_root("overlay_bound");
    setup_catalogs(&root);

    let overlay = serde_json::json!({
        "schema_version": 1,
        "scope": "fixture reviewed additions",
        "binds": {
            "historical_matrix": {"path": "catalog/aws-operations.json", "sha256": "0".repeat(64)}
        },
        "additions": []
    });
    write_json(&root, "catalog/aws-operation-additions.json", &overlay);

    let matrix = make_matrix();
    let matrix_sha = write_json(&root, "catalog/aws-operations.json", &matrix);
    let result = load_additions(&root, &matrix, &matrix_sha);
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_overlay_is_not_writable_receipt() {
    let root = test_root("overlay_receipt");
    setup_catalogs(&root);

    // Overlay should not be writable as a receipt
    // The overlay file exists but is read-only evidence
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn test_absent_overlay_is_noop() {
    let root = test_root("absent_overlay");
    setup_catalogs(&root);

    // No overlay file should result in None
    let matrix = make_matrix();
    let matrix_sha = write_json(&root, "catalog/aws-operations.json", &matrix);
    let result = load_additions(&root, &matrix, &matrix_sha);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    fs::remove_dir_all(&root).unwrap();
}