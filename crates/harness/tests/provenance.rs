//! Fixture provenance contracts: digests, catalog overlay and source set.

use serde_json::{Value, json};

use inventory_harness::provenance::{
    canonical_sha256, file_sha256, sha256_hex, source_fingerprint,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_known_bytes_when_hashed_then_lowercase_hex_matches_sha256() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn given_unsorted_object_when_canonicalized_then_digest_is_key_order_independent() {
    let first = json!({"b": 1, "a": [true, null, "x"]});
    let second = json!({"a": [true, null, "x"], "b": 1});
    assert_eq!(canonical_sha256(&first), canonical_sha256(&second));
    assert_eq!(canonical_sha256(&first).len(), 64);
}

#[test]
fn given_non_ascii_text_when_canonicalized_then_digest_matches_python_escaping() {
    // Python: hashlib.sha256(json.dumps({"k": "é"}, sort_keys=True,
    // separators=(",", ":")).encode()).hexdigest()
    assert_eq!(
        canonical_sha256(&json!({"k": "é"})),
        "9567d6e8f21bbdcbc2e0140bd733df64007ac8ed4af2329f39121e6349c3c86f"
    );
}

#[test]
fn given_workspace_root_when_fingerprinted_then_sources_and_manifests_are_covered() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fingerprint = source_fingerprint(&root)?;
    let files = fingerprint
        .get("files")
        .and_then(Value::as_array)
        .ok_or("fingerprint files")?;
    let names: Vec<&str> = files
        .iter()
        .filter_map(|row| row.get("file").and_then(Value::as_str))
        .collect();
    assert!(names.contains(&"Cargo.lock"));
    assert!(names.contains(&".cargo/config.toml"));
    assert!(names.iter().any(|name| name.starts_with("src/")));
    assert!(
        names
            .iter()
            .any(|name| name.starts_with("crates/engine/sql/"))
    );
    assert!(
        names
            .iter()
            .any(|name| name.starts_with("crates/harness/src/"))
    );
    assert!(
        names
            .iter()
            .all(|name| !name.contains("crates/harness/tests/")),
        "test-only sources are not compiled into the collector"
    );
    assert_eq!(
        fingerprint
            .get("sha256")
            .and_then(Value::as_str)
            .map(str::len),
        Some(64)
    );
    Ok(())
}

#[test]
fn given_changed_file_when_fingerprinted_then_digest_changes() -> TestResult {
    let root = tempfile::tempdir()?;
    let root = root.path();
    std::fs::create_dir_all(root.join("src"))?;
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n")?;
    for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
        std::fs::write(root.join(name), "frozen\n")?;
    }
    let before = source_fingerprint(root)?;
    std::fs::write(root.join("src/main.rs"), "fn main() { println!(\"x\"); }\n")?;
    let after = source_fingerprint(root)?;
    assert_ne!(
        before.get("sha256").and_then(Value::as_str),
        after.get("sha256").and_then(Value::as_str)
    );
    Ok(())
}

#[test]
fn given_runtime_overlay_when_merged_then_parent_workflow_is_additive() -> TestResult {
    let roots = json!({
        "resource_types": [{"type_name": "AWS::EKS::Cluster", "list_requires_model": false}],
        "evidence": {"roots": true}
    });
    let parents = json!({
        "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": true}],
        "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster"}],
        "evidence": {"parents": true}
    });
    let merged = inventory_harness::provenance::expected_runtime_catalog(&roots, &parents)?;
    let types = merged
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or("resource types")?;
    assert_eq!(types.len(), 2);
    let child = types
        .iter()
        .find(|row| row.get("type_name").and_then(Value::as_str) == Some("AWS::EKS::Addon"))
        .ok_or("parent-scoped child")?;
    assert_eq!(
        child
            .pointer("/parent_workflow/parent_type")
            .and_then(Value::as_str),
        Some("AWS::EKS::Cluster")
    );
    assert_eq!(
        merged
            .pointer("/parent_evidence/parents")
            .and_then(Value::as_bool),
        Some(true)
    );
    Ok(())
}

#[test]
fn given_unreviewed_root_prerequisite_when_merged_then_overlay_is_rejected() -> TestResult {
    let roots = json!({
        "resource_types": [{"type_name": "AWS::EKS::Cluster", "list_requires_model": true}]
    });
    let parents = json!({
        "resource_types": [{"type_name": "AWS::EKS::Addon", "list_requires_model": true}],
        "parent_plans": [{"type_name": "AWS::EKS::Addon", "parent_type": "AWS::EKS::Cluster"}],
        "evidence": {}
    });
    let error = inventory_harness::provenance::expected_runtime_catalog(&roots, &parents)
        .err()
        .ok_or("root prerequisite must be reviewed")?;
    assert!(error.to_string().contains("reviewed root prerequisites"));
    Ok(())
}

#[test]
fn given_file_when_hashed_then_matches_its_bytes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("payload");
    std::fs::write(&path, b"abc")?;
    assert_eq!(
        file_sha256(&path)?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    Ok(())
}
