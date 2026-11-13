//! Reviewed additive operation overlay.
//!
//! `catalog/aws-operation-additions.json` admits explicit SDK send callsites
//! discovered after the frozen operation matrix. The overlay never amends a
//! historical row and never supplies an emulator outcome: accepted statuses
//! are `unexercised` and `pending_execution`, both of which mean not exercised.
//! Every row is bound to the historical matrix digest, the SDK inventory
//! digest, its exact callsites, their source digests and a pinned SDK module.

mod row;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Value, json};

use crate::HarnessError;
use crate::provenance::sha256_hex;

use super::super::files::{array_field, object_field, read_json_audit};
use super::{INVENTORY, Inventory, OPERATION_ADDITIONS, OperationKey};

/// Maximum reviewed additions accepted in one overlay.
pub(crate) const MAX_ADDITIONS: usize = 512;

/// One reviewed callsite binding.
#[derive(Clone, Debug)]
pub(crate) struct AdditionSite {
    pub(crate) path: String,
    pub(crate) function: String,
    pub(crate) builder_line: u64,
    pub(crate) operation_line: u64,
    pub(crate) send_line: u64,
}

/// One reviewed addition and its pinned evidence.
#[derive(Clone, Debug)]
pub(crate) struct AdditionRow {
    pub(crate) id: String,
    pub(crate) key: OperationKey,
    pub(crate) service: String,
    pub(crate) crate_name: String,
    pub(crate) module: String,
    pub(crate) api_operation: String,
    pub(crate) status: String,
    pub(crate) module_sha256: String,
    pub(crate) sites: Vec<AdditionSite>,
    pub(crate) source_files: BTreeMap<String, String>,
}

/// Parsed overlay, or the explicit absent state.
#[derive(Clone, Debug)]
pub(crate) struct Additions {
    pub(crate) status: String,
    pub(crate) sha256: String,
    pub(crate) reviewed_at_utc: String,
    pub(crate) matrix_sha256: String,
    pub(crate) inventory_sha256: String,
    pub(crate) rows: Vec<AdditionRow>,
    pub(crate) keys: BTreeSet<OperationKey>,
}

/// Historical operation identities derived from matrix claims.
pub(crate) fn matrix_ids(
    inventory: &Inventory,
    claims: &BTreeMap<OperationKey, BTreeSet<String>>,
) -> BTreeSet<String> {
    claims
        .keys()
        .filter_map(|key| inventory.rows.get(key))
        .map(|row| format!("{}.{}", row.service, row.module))
        .collect()
}

impl Additions {
    /// No overlay document was present.
    pub(crate) fn absent() -> Self {
        Self {
            status: "absent".to_owned(),
            sha256: String::new(),
            reviewed_at_utc: String::new(),
            matrix_sha256: String::new(),
            inventory_sha256: String::new(),
            rows: Vec::new(),
            keys: BTreeSet::new(),
        }
    }

    /// Whether a reviewed overlay was loaded.
    pub(crate) fn is_present(&self) -> bool {
        self.status == "applied"
    }

    /// Read the overlay, or return the absent state when the file is missing.
    pub(crate) fn load(root: &Path, matrix_sha256: &str) -> Result<Self, HarnessError> {
        let path = root.join(OPERATION_ADDITIONS);
        if !path.try_exists()? {
            return Ok(Self::absent());
        }
        let (document, sha256) = read_json_audit(&path)?;
        Self::parse(&document, sha256, matrix_sha256)
    }

    fn parse(document: &Value, sha256: String, matrix_sha256: &str) -> Result<Self, HarnessError> {
        if document.get("schema_version").and_then(Value::as_u64) != Some(1) {
            return Err(HarnessError::invalid("invalid reviewed additions document"));
        }
        let binds = object_field(document, "binds")?;
        require_matrix_binding(binds, matrix_sha256)?;
        let inventory_sha256 = require_inventory_binding(binds)?;
        let review = object_field(document, "review")?;
        let reviewed_at_utc = review
            .get("reviewed_at_utc")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| HarnessError::invalid("invalid reviewed additions document"))?;
        let rows = parse_rows(document)?;
        let (rows, keys) = index_rows(rows)?;
        Ok(Self {
            status: "applied".to_owned(),
            sha256,
            reviewed_at_utc: reviewed_at_utc.to_owned(),
            matrix_sha256: matrix_sha256.to_owned(),
            inventory_sha256,
            rows,
            keys,
        })
    }

    /// Resolve every addition to the pinned inventory and reviewed sources.
    pub(crate) fn bind(
        &self,
        inventory: &Inventory,
        matrix_ids: &BTreeSet<String>,
        sources: &BTreeMap<String, String>,
    ) -> Result<(), HarnessError> {
        if !self.is_present() {
            return Ok(());
        }
        require_inventory_digest(self, inventory)?;
        for row in &self.rows {
            if matrix_ids.contains(&row.id) {
                return Err(HarnessError::invalid(format!(
                    "reviewed addition {} reuses a historical operation identity",
                    row.id
                )));
            }
            resolve(row, inventory)?;
            require_sources(row, sources)?;
        }
        Ok(())
    }

    /// Declared callsites with their reviewed origin and line binding.
    pub(crate) fn expected_callsites(&self) -> Vec<Value> {
        let mut rows = Vec::new();
        for row in &self.rows {
            for site in &row.sites {
                rows.push(json!({
                    "id": row.id,
                    "origin": "reviewed_addition",
                    "path": site.path,
                    "function": site.function,
                    "builder_line": site.builder_line,
                    "operation_line": site.operation_line,
                    "send_line": site.send_line,
                }));
            }
        }
        rows
    }
}

fn parse_rows(document: &Value) -> Result<Vec<AdditionRow>, HarnessError> {
    let values = array_field(document, "additions")?;
    if values.is_empty() || values.len() > MAX_ADDITIONS {
        return Err(HarnessError::invalid("invalid reviewed additions document"));
    }
    values.iter().map(row::parse).collect()
}

fn index_rows(
    rows: Vec<AdditionRow>,
) -> Result<(Vec<AdditionRow>, BTreeSet<OperationKey>), HarnessError> {
    let mut keys: BTreeSet<OperationKey> = BTreeSet::new();
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    for row in &rows {
        if !keys.insert(row.key.clone()) || !ids.insert(row.id.as_str()) {
            return Err(HarnessError::invalid("invalid reviewed additions document"));
        }
    }
    Ok((rows, keys))
}

fn require_matrix_binding(
    binds: &serde_json::Map<String, Value>,
    matrix_sha256: &str,
) -> Result<(), HarnessError> {
    let bound = binds
        .get("historical_matrix")
        .and_then(|value| value.get("sha256"))
        .and_then(Value::as_str);
    if bound != Some(matrix_sha256) {
        return Err(HarnessError::invalid(
            "reviewed additions bind a different historical operation matrix",
        ));
    }
    Ok(())
}

fn require_inventory_binding(
    binds: &serde_json::Map<String, Value>,
) -> Result<String, HarnessError> {
    let inventory = binds
        .get("operation_inventory")
        .and_then(Value::as_object)
        .ok_or_else(|| HarnessError::invalid("invalid reviewed additions document"))?;
    if inventory.get("path").and_then(Value::as_str) != Some(INVENTORY) {
        return Err(HarnessError::invalid("invalid reviewed additions document"));
    }
    let sha256 = inventory
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("invalid reviewed additions document"))?;
    if !row::hex64(sha256) {
        return Err(HarnessError::invalid("invalid reviewed additions document"));
    }
    Ok(sha256.to_owned())
}

fn require_inventory_digest(
    additions: &Additions,
    inventory: &Inventory,
) -> Result<(), HarnessError> {
    if additions.inventory_sha256 != inventory.sha256 {
        return Err(HarnessError::invalid(
            "reviewed additions bind a different SDK operation inventory",
        ));
    }
    Ok(())
}

fn resolve(row: &AdditionRow, inventory: &Inventory) -> Result<(), HarnessError> {
    let absent = || {
        HarnessError::invalid(format!(
            "reviewed addition {} is absent from {INVENTORY}",
            row.id
        ))
    };
    let (service, version) = inventory.by_crate.get(&row.crate_name).ok_or_else(absent)?;
    if *service != row.service || *version != row.key.1 {
        return Err(absent());
    }
    let operation = inventory.rows.get(&row.key).ok_or_else(absent)?;
    if operation.api_operation != row.api_operation || operation.module != row.module {
        return Err(absent());
    }
    Ok(())
}

fn require_sources(
    row: &AdditionRow,
    sources: &BTreeMap<String, String>,
) -> Result<(), HarnessError> {
    for (path, digest) in &row.source_files {
        let current = sources
            .get(path)
            .map(|source| sha256_hex(source.as_bytes()));
        if current.as_deref() != Some(digest.as_str()) {
            return Err(HarnessError::invalid(format!(
                "reviewed addition source changed since review: {path}"
            )));
        }
    }
    Ok(())
}
