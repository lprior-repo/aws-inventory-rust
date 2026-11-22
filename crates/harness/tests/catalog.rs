use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use inventory_harness::HarnessError;
use inventory_harness::catalog::{
    ARCHIVE, ARCHIVE_SHA256, CAPABILITY_MATRIX, PARENT_CATALOG, ROOT_CATALOG, UPSTREAM_MANIFEST,
    check, reproduce,
};
use inventory_harness::provenance::{canonical_sha256, file_sha256, sha256_hex};

const FILES: [&str; 6] = [
    ROOT_CATALOG,
    CAPABILITY_MATRIX,
    PARENT_CATALOG,
    "catalog/cloudcontrol-regional-additions.toml",
    ARCHIVE,
    UPSTREAM_MANIFEST,
];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), HarnessError> {
    for file in FILES {
        let destination = target.join(file);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source.join(file), destination)?;
    }
    Ok(())
}

fn replace(path: &Path, from: &str, to: &str) -> Result<(), HarnessError> {
    let text = std::fs::read_to_string(path)?;
    if text.matches(from).count() != 1 {
        return Err(HarnessError::invalid(format!(
            "expected exactly one {from} in {}",
            path.display()
        )));
    }
    std::fs::write(path, text.replace(from, to))?;
    Ok(())
}

fn report<'a>(
    reports: &'a [inventory_harness::catalog::Report],
    file: &str,
) -> Result<&'a inventory_harness::catalog::Report, HarnessError> {
    reports
        .iter()
        .find(|row| row.file == file)
        .ok_or_else(|| HarnessError::invalid(format!("missing report {file}")))
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing field {key}")))
}

#[test]
fn given_committed_tree_when_reproduced_then_documents_match_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let root_digest = file_sha256(&root.join(ROOT_CATALOG))?;
    assert_eq!(root_digest.len(), 64);
    assert_eq!(
        root_digest,
        "7d551d1ae5d32e83dc7bf45d5ef94fca35b0cecbc510cd5a3fe75aafa73300e4"
    );
    assert_eq!(file_sha256(&root.join(ARCHIVE))?, ARCHIVE_SHA256);
    let reports = reproduce(&root)?;
    for file in [ROOT_CATALOG, UPSTREAM_MANIFEST, PARENT_CATALOG] {
        let generated = report(&reports, file)?;
        assert_eq!(
            generated.contents.as_bytes(),
            std::fs::read(root.join(file))?.as_slice(),
            "{file} must reproduce byte for byte"
        );
    }
    let resources = report(&reports, ROOT_CATALOG)?;
    assert_eq!(sha256_hex(resources.contents.as_bytes()), root_digest);
    let parent: toml::Value = toml::from_str(&std::fs::read_to_string(root.join(PARENT_CATALOG))?)?;
    assert_eq!(
        parent
            .get("evidence")
            .and_then(|evidence| evidence.get("root_catalog_sha256"))
            .and_then(toml::Value::as_str),
        Some(root_digest.as_str())
    );
    check(&root)?;
    Ok(())
}

#[test]
fn given_committed_tree_when_checked_then_capability_partition_is_exact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let matrix: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(CAPABILITY_MATRIX))?)?;
    let rows = field(&matrix, "resource_types")?
        .as_array()
        .ok_or_else(|| HarnessError::invalid("capability rows"))?;
    assert_eq!(rows.len(), 1881);
    let mut exclusions: BTreeMap<String, usize> = BTreeMap::new();
    let mut selected = 0_usize;
    for row in rows {
        if field(row, "selected")?.as_bool() == Some(true) {
            selected = selected.saturating_add(1);
            assert_eq!(field(row, "exclusion")?.as_null(), Some(()));
        }
        if let Some(reason) = field(row, "exclusion")?.as_str() {
            let count = exclusions.entry(reason.to_string()).or_insert(0);
            *count = count.saturating_add(1);
        }
    }
    assert_eq!(selected, 397);
    assert_eq!(
        exclusions,
        BTreeMap::from([
            ("documented_parent_model_required".to_string(), 37),
            ("missing_read_or_list_handler".to_string(), 180),
            ("schema_parent_model_required".to_string(), 373),
            ("scope_unreviewed".to_string(), 894),
        ])
    );
    let catalog: Value = serde_json::from_str(&std::fs::read_to_string(root.join(ROOT_CATALOG))?)?;
    let types = field(&catalog, "resource_types")?
        .as_array()
        .ok_or_else(|| HarnessError::invalid("resource types"))?;
    assert_eq!(types.len(), 397);
    let services: std::collections::BTreeSet<&str> = types
        .iter()
        .filter_map(|row| row.get("service").and_then(Value::as_str))
        .collect();
    assert_eq!(services.len(), 76);
    let rules: usize = types
        .iter()
        .filter_map(|row| row.get("relationships").and_then(Value::as_array))
        .map(|items| items.len())
        .sum();
    assert_eq!(rules, 945);
    let snapshot = types
        .iter()
        .find(|row| row.get("type_name").and_then(Value::as_str) == Some("AWS::EC2::Snapshot"))
        .ok_or_else(|| HarnessError::invalid("EC2 snapshot descriptor"))?;
    assert_eq!(
        snapshot.get("arn_template").and_then(Value::as_str),
        Some("arn:{partition}:ec2:{region}::snapshot/{identifier}")
    );
    Ok(())
}

#[test]
fn given_first_two_hundred_rows_when_canonicalized_then_v04_projection_is_frozen()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let catalog: Value = serde_json::from_str(&std::fs::read_to_string(root.join(ROOT_CATALOG))?)?;
    let types = field(&catalog, "resource_types")?
        .as_array()
        .ok_or_else(|| HarnessError::invalid("resource types"))?;
    let projection: Vec<Value> = types
        .iter()
        .take(200)
        .map(|row| {
            json!({
                "type_name": row.get("type_name"),
                "view_name": row.get("view_name"),
                "columns": row.get("columns"),
            })
        })
        .collect();
    assert_eq!(
        canonical_sha256(&Value::Array(projection)),
        "6227d8c4eef92be7cc1593469efa613773d52cc53c06cca13e29f6200630b36f"
    );
    Ok(())
}

#[test]
fn given_tampered_root_row_when_checked_then_rejects_with_location()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    copy_tree(&workspace(), temporary.path())?;
    replace(
        &temporary.path().join(ROOT_CATALOG),
        "\"view_name\": \"aws_cc_ec2_instance\"",
        "\"view_name\": \"aws_cc_ec2_instance_tampered\"",
    )?;
    let error = check(temporary.path()).err();
    let message = error
        .ok_or_else(|| HarnessError::invalid("tampered catalog was accepted"))?
        .to_string();
    assert!(
        message.contains("catalog differs; regenerate with `inventory-harness catalog universal`")
    );
    assert!(message.contains("first difference at line"));
    Ok(())
}

#[test]
fn given_tampered_parent_row_when_checked_then_rejects_with_rust_message()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    copy_tree(&workspace(), temporary.path())?;
    replace(
        &temporary.path().join(PARENT_CATALOG),
        "kind = \"aws_eks_node_group\"",
        "kind = \"aws_eks_node_group_tampered\"",
    )?;
    let message = check(temporary.path())
        .err()
        .ok_or_else(|| HarnessError::invalid("tampered parent catalog was accepted"))?
        .to_string();
    assert!(
        message.contains(
            "parent catalog differs; regenerate with `inventory-harness catalog universal`"
        )
    );
    Ok(())
}

#[test]
fn given_tampered_capability_row_when_checked_then_rejects_after_evidence_projection()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    copy_tree(&workspace(), temporary.path())?;
    replace(
        &temporary.path().join(CAPABILITY_MATRIX),
        "\"selected_count\": 397",
        "\"selected_count\": 396",
    )?;
    let message = check(temporary.path())
        .err()
        .ok_or_else(|| HarnessError::invalid("tampered capability matrix was accepted"))?
        .to_string();
    assert!(message.contains(
        "capability matrix differs; regenerate with `inventory-harness catalog universal`"
    ));
    Ok(())
}

#[test]
fn given_tampered_archive_when_checked_then_rejects_checksum()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    copy_tree(&workspace(), temporary.path())?;
    let archive = temporary.path().join(ARCHIVE);
    let mut bytes = std::fs::read(&archive)?;
    let position = bytes.len().saturating_sub(1);
    if let Some(byte) = bytes.get_mut(position) {
        *byte = byte.wrapping_add(1);
    }
    std::fs::write(&archive, bytes)?;
    let message = check(temporary.path())
        .err()
        .ok_or_else(|| HarnessError::invalid("tampered archive was accepted"))?
        .to_string();
    assert_eq!(message, "upstream archive checksum mismatch");
    Ok(())
}
