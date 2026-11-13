//! Frozen catalog binding, historical operation identity and callsite parity.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::{canonical_sha256, expected_runtime_catalog, sha256_hex};

use super::calls::{CallSite, MAX_CALLS};
use super::files::{array_field, object_field, read_json_audit, text_field};

/// Both frozen Cloud Control catalogs and their additive runtime overlay.
pub struct Catalogs {
    /// Frozen root descriptor catalog.
    pub catalog: Value,
    /// Digest of the root catalog bytes.
    pub catalog_sha256: String,
    /// Frozen parent-dependent descriptor catalog.
    pub parents: Value,
    /// Digest of the parent catalog bytes.
    pub parent_sha256: String,
    /// Runtime overlay bound into executable catalogs.
    pub merged: Value,
}

/// Read both frozen catalogs and require their frozen binding.
pub fn catalogs(root: &Path) -> Result<Catalogs, HarnessError> {
    let (catalog, catalog_sha256) =
        read_json_audit(&root.join("catalog/cloudcontrol-resources.json"))?;
    let (parents, parent_sha256) =
        read_json_audit(&root.join("catalog/cloudcontrol-parent-resources.json"))?;
    let bound = parents
        .pointer("/evidence/root_catalog_sha256")
        .and_then(Value::as_str);
    if bound != Some(catalog_sha256.as_str()) {
        return Err(HarnessError::invalid(
            "parent catalog is not bound to the current frozen root catalog",
        ));
    }
    let merged = expected_runtime_catalog(&catalog, &parents)?;
    Ok(Catalogs {
        catalog,
        catalog_sha256,
        parents,
        parent_sha256,
        merged,
    })
}

/// Index historical operations by identity, refusing duplicates.
pub fn historical_operations(matrix: &Value) -> Result<BTreeMap<String, Value>, HarnessError> {
    let operations = array_field(matrix, "operations")?;
    let mut indexed = BTreeMap::new();
    for operation in operations {
        let id = text_field(operation, "id")?;
        if indexed.insert(id.to_owned(), operation.clone()).is_some() {
            return Err(HarnessError::invalid(
                "duplicate or excessive historical operation IDs",
            ));
        }
    }
    if indexed.len() != operations.len() || indexed.len() > MAX_CALLS {
        return Err(HarnessError::invalid(
            "duplicate or excessive historical operation IDs",
        ));
    }
    Ok(indexed)
}

/// Historical site identity: path, enclosing function and operation.
pub type SiteKey = (String, String, String);

fn site_key(row: &Value) -> Result<SiteKey, HarnessError> {
    Ok((
        text_field(row, "path")?.to_owned(),
        text_field(row, "function")?.to_owned(),
        text_field(row, "id")?.to_owned(),
    ))
}

fn bound_key(call: &CallSite) -> SiteKey {
    (call.path.clone(), call.function.clone(), call.id.clone())
}

fn site_repr(key: &SiteKey) -> String {
    format!("('{}', '{}', '{}')", key.0, key.1, key.2)
}

fn counts_repr(rows: &[(SiteKey, usize)]) -> String {
    let parts: Vec<String> = rows
        .iter()
        .map(|(key, count)| format!("{}: {count}", site_repr(key)))
        .collect();
    format!("{{{}}}", parts.join(", "))
}

/// Require the exact historical path/function/operation multiset.
///
/// Returns the historical rows indexed by site identity for position
/// comparison.
pub fn verify_callsites(
    expected: &[Value],
    calls: &[CallSite],
) -> Result<BTreeMap<SiteKey, Value>, HarnessError> {
    let (expected_order, expected_counts) = expected_sites(expected)?;
    let (actual_order, actual_counts) = actual_sites(calls);
    let missing = site_difference(&expected_order, &expected_counts, &actual_counts);
    let added = site_difference(&actual_order, &actual_counts, &expected_counts);
    require_parity(&missing, &added)?;
    index_sites(expected)
}

/// Historical site order and multiplicity.
fn expected_sites(
    rows: &[Value],
) -> Result<(Vec<SiteKey>, BTreeMap<SiteKey, usize>), HarnessError> {
    let mut order: Vec<SiteKey> = Vec::new();
    let mut counts: BTreeMap<SiteKey, usize> = BTreeMap::new();
    for row in rows {
        tally_site(&mut order, &mut counts, site_key(row)?);
    }
    Ok((order, counts))
}

/// Current callsite order and multiplicity.
fn actual_sites(calls: &[CallSite]) -> (Vec<SiteKey>, BTreeMap<SiteKey, usize>) {
    let mut order: Vec<SiteKey> = Vec::new();
    let mut counts: BTreeMap<SiteKey, usize> = BTreeMap::new();
    for call in calls {
        tally_site(&mut order, &mut counts, bound_key(call));
    }
    (order, counts)
}

fn tally_site(order: &mut Vec<SiteKey>, counts: &mut BTreeMap<SiteKey, usize>, key: SiteKey) {
    if !counts.contains_key(&key) {
        order.push(key.clone());
    }
    let count = counts.entry(key).or_insert(0);
    *count = count.saturating_add(1);
}

/// Sites whose count in `self_counts` exceeds `other_counts`.
fn site_difference(
    order: &[SiteKey],
    self_counts: &BTreeMap<SiteKey, usize>,
    other_counts: &BTreeMap<SiteKey, usize>,
) -> Vec<(SiteKey, usize)> {
    order
        .iter()
        .filter_map(|key| {
            let expected = self_counts.get(key).copied().unwrap_or(0);
            let actual = other_counts.get(key).copied().unwrap_or(0);
            (expected > actual).then(|| (key.clone(), expected.saturating_sub(actual)))
        })
        .collect()
}

fn require_parity(
    missing: &[(SiteKey, usize)],
    added: &[(SiteKey, usize)],
) -> Result<(), HarnessError> {
    if !missing.is_empty() || !added.is_empty() {
        return Err(HarnessError::invalid(format!(
            "SDK operation boundary changes require review: missing={} added={}",
            counts_repr(missing),
            counts_repr(added)
        )));
    }
    Ok(())
}

fn index_sites(rows: &[Value]) -> Result<BTreeMap<SiteKey, Value>, HarnessError> {
    let mut indexed = BTreeMap::new();
    for row in rows {
        indexed.insert(site_key(row)?, row.clone());
    }
    if indexed.len() != rows.len() {
        return Err(HarnessError::invalid(
            "historical callsites are ambiguous within one function",
        ));
    }
    Ok(indexed)
}

/// Per-file source digests against both preserved receipts.
pub(crate) fn source_rows(
    matrix: &Value,
    previous: &Value,
    sources: &BTreeMap<String, String>,
) -> Result<Vec<Value>, HarnessError> {
    let extraction = object_field(matrix, "extraction")?;
    let old_hashes = extraction
        .get("source_sha256")
        .and_then(Value::as_object)
        .ok_or_else(|| HarnessError::invalid("missing extraction.source_sha256"))?;
    let previous_hashes = object_field(previous, "all_current_aws_source_sha256")?;
    Ok(sources
        .iter()
        .map(|(path, source)| source_row(path, source, old_hashes, previous_hashes))
        .collect())
}

fn source_row(
    path: &str,
    source: &str,
    old_hashes: &serde_json::Map<String, Value>,
    previous_hashes: &serde_json::Map<String, Value>,
) -> Value {
    let digest = sha256_hex(source.as_bytes());
    let historical = old_hashes.get(path).cloned().unwrap_or(Value::Null);
    let v04 = previous_hashes.get(path).cloned().unwrap_or(Value::Null);
    serde_json::json!({
        "path": path,
        "sha256": digest,
        "historical_sha256": historical,
        "v04_sha256": v04,
        "changed_since_historical_matrix": historical != Value::String(digest.clone()),
        "changed_since_v04_reconciliation": v04 != Value::String(digest),
    })
}

/// Runtime catalog canonical digest for the provider dimension.
pub(crate) fn runtime_digest(merged: &Value) -> String {
    canonical_sha256(merged)
}

/// Validate descriptor identity and return the type-name set.
pub(crate) fn descriptor_names(
    merged: &Value,
) -> Result<(Vec<Value>, BTreeSet<String>), HarnessError> {
    let descriptors = array_field(merged, "resource_types")?;
    let mut names = BTreeSet::new();
    let mut duplicate = false;
    for descriptor in descriptors {
        if !names.insert(text_field(descriptor, "type_name")?.to_owned()) {
            duplicate = true;
        }
    }
    if descriptors.is_empty() || descriptors.len() > MAX_CALLS || duplicate {
        return Err(HarnessError::invalid(
            "invalid Cloud Control descriptor set",
        ));
    }
    Ok((descriptors.clone(), names))
}
