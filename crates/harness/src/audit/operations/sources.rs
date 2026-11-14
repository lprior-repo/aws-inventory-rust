//! Recorded emulator outcomes used to qualify operation coverage claims.

mod receipt;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::sha256_hex;

use super::super::files::{array_field, read_json_audit, text_field};
use super::additions::{self, Additions};
use super::{INVENTORY, Inventory, OPERATION_MATRIX, OperationKey, OperationRow, RECONCILIATION};

/// Recorded outcomes and the document that supplied them.
pub(crate) struct StatusEvidence {
    pub(crate) statuses: BTreeMap<OperationKey, String>,
    pub(crate) claims: BTreeMap<OperationKey, BTreeSet<String>>,
    pub(crate) reviewed_files: usize,
    pub(crate) reviewed_changed: usize,
    pub(crate) status_source: String,
    pub(crate) status_source_path: String,
    pub(crate) matrix_sha256: String,
    pub(crate) matrix_execution: String,
    pub(crate) reconciliation_status: String,
    pub(crate) reconciliation_sha256: Option<String>,
    pub(crate) additions: Additions,
}

/// Source files the matrix reviewed and how many changed since.
fn reviewed_hashes(document: &Value, sources: &BTreeMap<String, String>) -> (usize, usize) {
    let mut reviewed_files = 0_usize;
    let mut reviewed_changed = 0_usize;
    if let Some(hashes) = document
        .pointer("/extraction/source_sha256")
        .and_then(Value::as_object)
    {
        for (path, digest) in hashes {
            reviewed_files = reviewed_files.saturating_add(1);
            let current = sources
                .get(path)
                .map(|source| sha256_hex(source.as_bytes()));
            if current.as_deref() != digest.as_str() {
                reviewed_changed = reviewed_changed.saturating_add(1);
            }
        }
    }
    (reviewed_files, reviewed_changed)
}

fn operation_claims(row: &Value) -> Result<BTreeSet<String>, HarnessError> {
    let mut callsites: BTreeSet<String> = BTreeSet::new();
    for call in array_field(row, "source_calls")? {
        callsites.insert(format!(
            "reviewed:{}:{}",
            text_field(call, "path")?,
            text_field(call, "function")?
        ));
    }
    Ok(callsites)
}

fn recorded_version(row: &Value) -> Result<&str, HarnessError> {
    // Versionless fixture rows can match only versionless inventory rows, never an active SDK.
    match row.get("sdk_version") {
        Some(_) => text_field(row, "sdk_version"),
        None => Ok(""),
    }
}

/// Inventory row and key for one recorded matrix operation.
fn recorded_key<'a>(
    row: &Value,
    inventory: &'a Inventory,
) -> Result<(OperationKey, &'a OperationRow), HarnessError> {
    let id = text_field(row, "id")?;
    let crate_name = text_field(row, "sdk_crate")?;
    let module = text_field(row, "sdk_operation")?;
    let (service, _) = inventory.by_crate.get(crate_name).ok_or_else(|| {
        HarnessError::invalid(format!("recorded operation {id} names an unknown crate"))
    })?;
    let version = recorded_version(row)?;
    let key = (crate_name.to_owned(), version.to_owned(), module.to_owned());
    let operation = inventory.rows.get(&key).ok_or_else(|| {
        HarnessError::invalid(format!(
            "recorded operation {id} at {crate_name} {version} is absent from {INVENTORY}"
        ))
    })?;
    if format!("{service}.{}", operation.module) != id {
        return Err(HarnessError::invalid(format!(
            "recorded operation {id} disagrees with its crate/module"
        )));
    }
    Ok((key, operation))
}

fn provider_boundary(row: &Value) -> bool {
    row.get("credential_provider_boundary")
        .and_then(Value::as_bool)
        == Some(true)
}

/// Source review follows the active callsite; execution evidence keeps its recorded version.
fn current_claim_key(
    key: &OperationKey,
    inventory: &Inventory,
) -> Result<OperationKey, HarnessError> {
    let (_, version) = inventory
        .by_crate
        .get(&key.0)
        .ok_or_else(|| HarnessError::invalid("reviewed callsite names an unknown SDK crate"))?;
    let current = (key.0.clone(), version.clone(), key.2.clone());
    if !inventory.rows.contains_key(&current) {
        return Err(HarnessError::invalid(
            "reviewed callsite has no active SDK operation",
        ));
    }
    Ok(current)
}

fn record_operation(
    row: &Value,
    inventory: &Inventory,
    statuses: &mut BTreeMap<OperationKey, String>,
    claims: &mut BTreeMap<OperationKey, BTreeSet<String>>,
) -> Result<(), HarnessError> {
    let (key, _operation) = recorded_key(row, inventory)?;
    claims
        .entry(current_claim_key(&key, inventory)?)
        .or_default()
        .extend(operation_claims(row)?);
    let Some(coverage) = row.get("emulator_coverage") else {
        return Ok(());
    };
    if provider_boundary(coverage) {
        return Ok(());
    }
    statuses.insert(key, text_field(coverage, "status")?.to_owned());
    Ok(())
}

fn matrix_execution(document: &Value) -> String {
    document
        .pointer("/emulator_run/evidence_directory")
        .and_then(Value::as_str)
        .unwrap_or("unspecified")
        .to_owned()
}

fn matrix_evidence(
    statuses: BTreeMap<OperationKey, String>,
    claims: BTreeMap<OperationKey, BTreeSet<String>>,
    reviewed: (usize, usize),
    document: &Value,
    sha256: String,
) -> StatusEvidence {
    StatusEvidence {
        statuses,
        claims,
        reviewed_files: reviewed.0,
        reviewed_changed: reviewed.1,
        status_source: "operation_matrix".to_owned(),
        status_source_path: OPERATION_MATRIX.to_owned(),
        matrix_sha256: sha256,
        matrix_execution: matrix_execution(document),
        reconciliation_status: "absent".to_owned(),
        reconciliation_sha256: None,
        additions: Additions::absent(),
    }
}

fn matrix_statuses(
    root: &Path,
    inventory: &Inventory,
    sources: &BTreeMap<String, String>,
) -> Result<StatusEvidence, HarnessError> {
    let (document, sha256) = read_json_audit(&root.join(OPERATION_MATRIX))?;
    let reviewed = reviewed_hashes(&document, sources);
    let mut statuses: BTreeMap<OperationKey, String> = BTreeMap::new();
    let mut claims: BTreeMap<OperationKey, BTreeSet<String>> = BTreeMap::new();
    for row in array_field(&document, "operations")? {
        record_operation(row, inventory, &mut statuses, &mut claims)?;
    }
    if statuses.is_empty() {
        return Err(HarnessError::invalid(
            "operation matrix records no emulator outcomes",
        ));
    }
    Ok(matrix_evidence(
        statuses, claims, reviewed, &document, sha256,
    ))
}

fn reconciliation_evidence(
    statuses: BTreeMap<OperationKey, String>,
    sha256: String,
    fallback: StatusEvidence,
) -> StatusEvidence {
    StatusEvidence {
        statuses,
        status_source: "reconciliation".to_owned(),
        status_source_path: RECONCILIATION.to_owned(),
        reconciliation_status: "applied".to_owned(),
        reconciliation_sha256: Some(sha256),
        ..fallback
    }
}

fn reconciliation_statuses(
    root: &Path,
    inventory: &Inventory,
    sources: &BTreeMap<String, String>,
    fallback: StatusEvidence,
) -> Result<StatusEvidence, HarnessError> {
    let path = root.join(RECONCILIATION);
    if !path.try_exists()? {
        return Ok(fallback);
    }
    let (document, sha256) = read_json_audit(&path)?;
    if !receipt::source_matches(&document, sources)? {
        return Ok(stale_reconciliation(
            fallback,
            sha256,
            "stale_source_closure",
        ));
    }
    if !receipt::sdk_matches(root, &document, inventory)? {
        return Ok(stale_reconciliation(fallback, sha256, "stale_sdk_closure"));
    }
    let statuses = receipt::statuses(&document, inventory)?;
    Ok(reconciliation_evidence(statuses, sha256, fallback))
}

fn stale_reconciliation(fallback: StatusEvidence, sha256: String, status: &str) -> StatusEvidence {
    StatusEvidence {
        reconciliation_status: status.to_owned(),
        reconciliation_sha256: Some(sha256),
        ..fallback
    }
}

fn additions_evidence(
    root: &Path,
    inventory: &Inventory,
    sources: &BTreeMap<String, String>,
    mut evidence: StatusEvidence,
) -> Result<StatusEvidence, HarnessError> {
    let additions = additions::Additions::load(root, &evidence.matrix_sha256)?;
    if additions.is_present() {
        additions.bind(
            inventory,
            &additions::matrix_ids(inventory, &evidence.claims),
            sources,
        )?;
    }
    evidence.additions = additions;
    Ok(evidence)
}

/// Load the newest applicable recorded outcomes for asserted operations.
pub(crate) fn statuses(
    root: &Path,
    inventory: &Inventory,
    sources: &BTreeMap<String, String>,
) -> Result<StatusEvidence, HarnessError> {
    let fallback = matrix_statuses(root, inventory, sources)?;
    let bound = reconciliation_statuses(root, inventory, sources, fallback)?;
    additions_evidence(root, inventory, sources, bound)
}
