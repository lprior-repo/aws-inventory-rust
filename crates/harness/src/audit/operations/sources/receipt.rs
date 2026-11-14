//! Reconciliation receipts apply only to their exact source and SDK module closure.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use super::super::{INVENTORY, Inventory, OperationKey};
use super::{
    HarnessError, array_field, provider_boundary, read_json_audit, sha256_hex, text_field,
};

pub(super) fn source_matches(
    document: &Value,
    sources: &BTreeMap<String, String>,
) -> Result<bool, HarnessError> {
    let paths = array_field(document, "aws_source_files")?.iter().try_fold(
        Some(BTreeSet::new()),
        |seen, row| -> Result<_, HarnessError> {
            let path = text_field(row, "path")?;
            let digest = text_field(row, "sha256")?;
            let current = sources
                .get(path)
                .map(|source| sha256_hex(source.as_bytes()));
            Ok(seen.and_then(|mut paths| {
                if current.as_deref() != Some(digest) || !paths.insert(path) {
                    return None;
                }
                Some(paths)
            }))
        },
    )?;
    Ok(paths.is_some_and(|paths| paths.len() == sources.len()))
}

fn key(pin: &Value, inventory: &Inventory) -> Result<OperationKey, HarnessError> {
    let id = text_field(pin, "id")?;
    let (service, module) = id
        .split_once('.')
        .ok_or_else(|| HarnessError::invalid(format!("invalid operation identity {id}")))?;
    let crate_name = text_field(pin, "sdk_crate")?;
    let version = text_field(pin, "sdk_version")?;
    let key = (crate_name.to_owned(), version.to_owned(), module.to_owned());
    let operation = inventory.rows.get(&key).ok_or_else(|| {
        HarnessError::invalid(format!(
            "recorded operation {id} at {crate_name} {version} is absent from {INVENTORY}"
        ))
    })?;
    if operation.service != service || operation.api_operation != text_field(pin, "api_operation")?
    {
        return Err(HarnessError::invalid(format!(
            "reconciliation SDK pin disagrees with {id}"
        )));
    }
    Ok(key)
}

fn published_digest<'a>(published: &'a Value, key: &OperationKey) -> Result<&'a str, HarnessError> {
    let entry = array_field(published, "crates")?
        .iter()
        .find(|entry| {
            entry.get("name").and_then(Value::as_str) == Some(key.0.as_str())
                && entry.get("version").and_then(Value::as_str) == Some(key.1.as_str())
        })
        .ok_or_else(|| HarnessError::invalid("receipt SDK crate is absent from inventory"))?;
    let operation = array_field(entry, "operations")?
        .iter()
        .find(|operation| operation.get("module").and_then(Value::as_str) == Some(key.2.as_str()))
        .ok_or_else(|| HarnessError::invalid("receipt SDK module is absent from inventory"))?;
    text_field(operation, "source_sha256")
}

fn pin_matches(
    pin: &Value,
    inventory: &Inventory,
    published: &Value,
) -> Result<bool, HarnessError> {
    let key = key(pin, inventory)?;
    let active = inventory
        .by_crate
        .get(&key.0)
        .is_some_and(|(_, version)| version == &key.1);
    Ok(active && published_digest(published, &key)? == text_field(pin, "module_sha256")?)
}

pub(super) fn sdk_matches(
    root: &Path,
    document: &Value,
    inventory: &Inventory,
) -> Result<bool, HarnessError> {
    let (published, _) = read_json_audit(&root.join(INVENTORY))?;
    let pins = array_field(document, "pinned_sdk_operation_modules")?;
    if pins.is_empty() {
        return Ok(false);
    }
    pins.iter().try_fold(BTreeSet::new(), |mut ids, pin| {
        let id = text_field(pin, "id")?;
        if !ids.insert(id) {
            return Err(HarnessError::invalid(format!(
                "duplicate reconciliation SDK pin {id}"
            )));
        }
        Ok(ids)
    })?;
    pins.iter().try_fold(true, |matches, pin| {
        Ok(matches && pin_matches(pin, inventory, &published)?)
    })
}

fn outcome(
    row: &Value,
    pins: &[Value],
    inventory: &Inventory,
) -> Result<Option<(OperationKey, String)>, HarnessError> {
    if provider_boundary(row) {
        return Ok(None);
    }
    let id = text_field(row, "id")?;
    let pin = pins
        .iter()
        .find(|pin| pin.get("id").and_then(Value::as_str) == Some(id))
        .ok_or_else(|| {
            HarnessError::invalid(format!(
                "reconciliation outcome {id} has no pinned SDK operation"
            ))
        })?;
    Ok(Some((
        key(pin, inventory)?,
        text_field(row, "status")?.to_owned(),
    )))
}

pub(super) fn statuses(
    document: &Value,
    inventory: &Inventory,
) -> Result<BTreeMap<OperationKey, String>, HarnessError> {
    let results = document
        .pointer("/emulator_evidence/operation_results")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("reconciliation receipt lacks operation results"))?;
    let pins = array_field(document, "pinned_sdk_operation_modules")?;
    let statuses = collect_statuses(results, pins, inventory)?;
    if statuses.is_empty() {
        return Err(HarnessError::invalid(
            "reconciliation receipt records no operation outcomes",
        ));
    }
    Ok(statuses)
}

fn collect_statuses(
    results: &[Value],
    pins: &[Value],
    inventory: &Inventory,
) -> Result<BTreeMap<OperationKey, String>, HarnessError> {
    results
        .iter()
        .try_fold(BTreeMap::new(), |mut statuses, row| {
            if let Some((key, status)) = outcome(row, pins, inventory)?
                && statuses.insert(key, status).is_some()
            {
                return Err(HarnessError::invalid(
                    "duplicate reconciliation operation outcome",
                ));
            }
            Ok(statuses)
        })
}
