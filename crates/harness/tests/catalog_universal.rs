//! Universal catalog contracts: full archive coverage, partition sums, parity.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use inventory_harness::HarnessError;
use inventory_harness::catalog::universal::{self, UNIVERSAL_CATALOG};
use inventory_harness::catalog::{ARCHIVE, ARCHIVE_SHA256, CAPABILITY_MATRIX, PARENT_CATALOG};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing field {key}")))
}

fn parent_reason(reason: &str) -> bool {
    matches!(
        reason,
        "schema_parent_model_required" | "documented_parent_model_required"
    )
}

fn copy_catalog_inputs(source: &Path, target: &Path) -> Result<(), HarnessError> {
    for file in [CAPABILITY_MATRIX, PARENT_CATALOG, ARCHIVE] {
        let destination = target.join(file);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source.join(file), destination)?;
    }
    Ok(())
}

#[test]
fn given_archive_when_generated_then_every_matrix_type_is_classified()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let document = universal::document(&root)?.to_serde()?;
    let source = field(&document, "source")?;
    assert_eq!(field(source, "archive")?.as_str(), Some(ARCHIVE));
    assert_eq!(field(source, "sha256")?.as_str(), Some(ARCHIVE_SHA256));
    assert_eq!(field(source, "schema_count")?.as_u64(), Some(1881));

    let summary = field(&document, "summary")?;
    let total = field(summary, "total")?
        .as_u64()
        .ok_or_else(|| HarnessError::invalid("summary total"))?;
    let list = field(summary, "list_capable")?
        .as_u64()
        .ok_or_else(|| HarnessError::invalid("summary list_capable"))?;
    let parent = field(summary, "parent_required")?
        .as_u64()
        .ok_or_else(|| HarnessError::invalid("summary parent_required"))?;
    let unsupported = field(summary, "unsupported")?
        .as_u64()
        .ok_or_else(|| HarnessError::invalid("summary unsupported"))?;
    assert_eq!(total, 1881);
    assert_eq!(
        total,
        list.saturating_add(parent).saturating_add(unsupported)
    );

    let rows = field(&document, "resource_types")?
        .as_array()
        .ok_or_else(|| HarnessError::invalid("universal rows"))?;
    assert_eq!(u64::try_from(rows.len())?, total);

    let matrix: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(CAPABILITY_MATRIX))?)?;
    let matrix_rows = field(&matrix, "resource_types")?
        .as_array()
        .ok_or_else(|| HarnessError::invalid("matrix rows"))?;
    assert_eq!(matrix_rows.len(), 1881);
    let mut expected: BTreeMap<String, (&'static str, Option<String>)> = BTreeMap::new();
    let mut expected_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for row in matrix_rows {
        let name = field(row, "type_name")?
            .as_str()
            .ok_or_else(|| HarnessError::invalid("matrix type_name"))?
            .to_string();
        let exclusion = field(row, "exclusion")?.as_str().map(str::to_string);
        let listing = field(row, "list_supported")?
            .as_bool()
            .ok_or_else(|| HarnessError::invalid("matrix list_supported"))?;
        let capability = if exclusion.as_deref().is_some_and(parent_reason) {
            "parent"
        } else if listing {
            "list"
        } else {
            "unsupported"
        };
        let slot = expected_counts.entry(capability).or_insert(0);
        *slot = slot.saturating_add(1);
        expected.insert(name, (capability, exclusion));
    }
    assert_eq!(
        u64::try_from(expected_counts.get("list").copied().unwrap_or(0))?,
        list
    );
    assert_eq!(
        u64::try_from(expected_counts.get("parent").copied().unwrap_or(0))?,
        parent
    );
    assert_eq!(
        u64::try_from(expected_counts.get("unsupported").copied().unwrap_or(0))?,
        unsupported
    );

    let mut previous: Option<&str> = None;
    for row in rows {
        let name = field(row, "type_name")?
            .as_str()
            .ok_or_else(|| HarnessError::invalid("universal type_name"))?;
        let (capability, exclusion) = expected
            .get(name)
            .ok_or_else(|| HarnessError::invalid(format!("matrix lacks {name}")))?;
        assert_eq!(
            field(row, "capability")?.as_str(),
            Some(*capability),
            "{name}"
        );
        assert_eq!(
            field(row, "exclusion")?.as_str(),
            exclusion.as_deref(),
            "{name}"
        );
        assert_eq!(
            field(row, "schema_sha256")?.as_str().map(str::len),
            Some(64),
            "{name}"
        );
        assert_eq!(field(row, "visibility")?.as_str(), Some("PUBLIC"), "{name}");
        if !field(row, "parent_type")?.is_null() {
            assert_eq!(*capability, "parent", "{name}");
            assert!(
                field(row, "parent_type")?
                    .as_str()
                    .is_some_and(|parent| parent.starts_with("AWS::")),
                "{name}"
            );
        }
        if let Some(previous) = previous {
            assert!(previous < name, "{name} must be sorted after {previous}");
        }
        previous = Some(name);
    }
    Ok(())
}

#[test]
fn given_committed_catalog_when_regenerated_then_bytes_are_identical()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let first = universal::rendered(&root)?;
    assert_eq!(first, universal::rendered(&root)?);
    assert!(first.ends_with('\n'), "catalog must end with a newline");
    assert_eq!(
        first.as_bytes(),
        std::fs::read(root.join(UNIVERSAL_CATALOG))?.as_slice()
    );
    universal::check(&root)?;
    Ok(())
}

#[test]
fn given_modified_catalog_when_checked_then_rejects_with_location()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace();
    let temporary = tempfile::tempdir()?;
    copy_catalog_inputs(&root, temporary.path())?;
    let expected = universal::rendered(&root)?;
    let tampered = expected.replacen(
        "\"capability\": \"list\"",
        "\"capability\": \"unsupported\"",
        1,
    );
    assert_ne!(tampered, expected);
    std::fs::write(temporary.path().join(UNIVERSAL_CATALOG), &tampered)?;
    let message = universal::check(temporary.path())
        .err()
        .ok_or_else(|| HarnessError::invalid("tampered universal catalog was accepted"))?
        .to_string();
    assert!(message.contains("universal catalog differs"));
    assert!(message.contains("inventory-harness catalog universal"));
    assert!(message.contains("first difference at line"));
    std::fs::write(temporary.path().join(UNIVERSAL_CATALOG), &expected)?;
    universal::check(temporary.path())?;
    Ok(())
}
