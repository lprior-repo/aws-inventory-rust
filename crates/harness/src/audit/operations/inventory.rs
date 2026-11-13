//! Load the published operation inventory and resolve active crate versions.

mod lock;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::sha256_hex;

use super::super::calls::CallSite;
use super::super::files::{
    MAX_INPUT_BYTES, array_field, decode_utf8, object_field, read_bytes, read_json_audit,
    slash_path, text_field,
};
use super::super::inventory::source_inventory;
use super::{CRATE_PREFIX, Claim, INVENTORY, Inventory, OperationKey, OperationRow};
use lock::{lockfile, resolve_active};

const MAX_SOURCE_FILES: usize = 4_096;
const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

/// Published operation rows keyed by crate, version and module.
type OperationRows = BTreeMap<OperationKey, OperationRow>;
/// Candidate crate versions keyed by service namespace.
type VersionMap = BTreeMap<String, BTreeSet<(String, String)>>;

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), HarnessError> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn rust_paths(root: &Path) -> Result<Vec<PathBuf>, HarnessError> {
    let mut paths = Vec::new();
    collect_rust_files(&root.join("crates/aws/src"), &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn read_source(
    root: &Path,
    path: &Path,
    count: usize,
    total: &mut u64,
) -> Result<(String, String), HarnessError> {
    if count >= MAX_SOURCE_FILES || fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(HarnessError::invalid("unsupported AWS source inventory"));
    }
    let raw = read_bytes(path, MAX_INPUT_BYTES)?;
    *total = total.saturating_add(u64::try_from(raw.len()).unwrap_or(u64::MAX));
    if *total > MAX_SOURCE_BYTES {
        return Err(HarnessError::invalid("AWS source byte budget exceeded"));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| HarnessError::invalid("AWS source escaped the workspace root"))?;
    Ok((slash_path(relative), decode_utf8(raw, path)?))
}

/// Read current AWS sources without the strict callsite scanner, which fails
/// closed on call shapes it cannot resolve.
pub(crate) fn source_texts(root: &Path) -> Result<BTreeMap<String, String>, HarnessError> {
    let paths = rust_paths(root)?;
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    let mut total = 0_u64;
    for path in &paths {
        let (name, text) = read_source(root, path, sources.len(), &mut total)?;
        sources.insert(name, text);
    }
    if sources.is_empty() {
        return Err(HarnessError::invalid("empty AWS source inventory"));
    }
    Ok(sources)
}

/// Result of scanning current collector callsites.
pub(crate) struct Scan {
    pub(crate) claims: BTreeMap<OperationKey, Claim>,
    pub(crate) files: usize,
    pub(crate) callsites: usize,
    pub(crate) nonempty: usize,
    pub(crate) sha256: String,
}

/// One published crate row and its derived service namespace.
struct CrateIdentity<'a> {
    name: &'a str,
    service: &'a str,
    version: String,
}

fn crate_identity(entry: &Value) -> Result<CrateIdentity<'_>, HarnessError> {
    let name = text_field(entry, "name")?;
    let service = name
        .strip_prefix(CRATE_PREFIX)
        .filter(|suffix| !suffix.is_empty())
        .ok_or_else(|| HarnessError::invalid(format!("unrecognized SDK crate name {name}")))?;
    let version = entry
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    Ok(CrateIdentity {
        name,
        service,
        version,
    })
}

fn operation_row(
    operation: &Value,
    identity: &CrateIdentity<'_>,
) -> Result<OperationRow, HarnessError> {
    Ok(OperationRow {
        service: identity.service.to_owned(),
        crate_name: identity.name.to_owned(),
        version: identity.version.clone(),
        module: text_field(operation, "module")?.to_owned(),
        api_operation: text_field(operation, "api_operation")?.to_owned(),
    })
}

fn load_operations(
    entry: &Value,
    identity: &CrateIdentity<'_>,
    rows: &mut OperationRows,
) -> Result<(), HarnessError> {
    for operation in array_field(entry, "operations")? {
        let row = operation_row(operation, identity)?;
        if rows.insert(row.key(), row).is_some() {
            return Err(HarnessError::invalid(format!(
                "duplicate operation module in {} {}",
                identity.name, identity.version
            )));
        }
    }
    Ok(())
}

fn load_crates(crates: &[Value]) -> Result<(OperationRows, VersionMap), HarnessError> {
    let mut rows: OperationRows = BTreeMap::new();
    let mut versions: VersionMap = BTreeMap::new();
    for entry in crates {
        let identity = crate_identity(entry)?;
        versions
            .entry(identity.service.to_owned())
            .or_default()
            .insert((identity.name.to_owned(), identity.version.clone()));
        load_operations(entry, &identity, &mut rows)?;
    }
    Ok((rows, versions))
}

fn check_summary(document: &Value, crates: usize, operations: usize) -> Result<(), HarnessError> {
    let summary = object_field(document, "summary")?;
    if summary.get("crates").and_then(Value::as_u64) != u64::try_from(crates).ok()
        || summary.get("operations").and_then(Value::as_u64) != u64::try_from(operations).ok()
    {
        return Err(HarnessError::invalid(
            "SDK operation inventory summary differs from its rows",
        ));
    }
    Ok(())
}

/// Read and validate the published inventory.
pub(crate) fn load(root: &Path) -> Result<Inventory, HarnessError> {
    let (document, sha256) = read_json_audit(&root.join(INVENTORY))?;
    if document.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(HarnessError::invalid("unsupported SDK operation inventory"));
    }
    let crates = array_field(&document, "crates")?;
    let (rows, versions) = load_crates(crates)?;
    check_summary(&document, crates.len(), rows.len())?;
    if rows.is_empty() {
        return Err(HarnessError::invalid("empty SDK operation inventory"));
    }
    let (active, by_crate) = resolve_active(&versions, &lockfile(root)?)?;
    Ok(Inventory {
        rows,
        active,
        by_crate,
        sha256,
        crates: crates.len(),
    })
}

/// Digest closure of every scanned source path and body.
fn closure_text(sources: &BTreeMap<String, String>) -> String {
    let mut closure = String::new();
    for (path, source) in sources {
        closure.push_str(path);
        closure.push(':');
        closure.push_str(&sha256_hex(source.as_bytes()));
        closure.push('\n');
    }
    closure
}

fn claim_key(call: &CallSite, inventory: &Inventory) -> Result<OperationKey, HarnessError> {
    let (crate_name, version) = inventory.active.get(&call.sdk_service).ok_or_else(|| {
        HarnessError::invalid(format!(
            "callsite {}:{} names unknown SDK service {}",
            call.path, call.function, call.sdk_service
        ))
    })?;
    let key = (
        crate_name.clone(),
        version.clone(),
        call.sdk_operation.clone(),
    );
    let row = inventory.rows.get(&key).ok_or_else(|| {
        HarnessError::invalid(format!(
            "callsite {}:{} calls {}.{}, which is absent from {INVENTORY}",
            call.path, call.function, crate_name, call.sdk_operation
        ))
    })?;
    if row.service != call.sdk_service {
        return Err(HarnessError::invalid(
            "callsite SDK namespace disagrees with the operation inventory",
        ));
    }
    Ok(key)
}

fn collect_claims(
    calls: &[CallSite],
    inventory: &Inventory,
) -> Result<BTreeMap<OperationKey, Claim>, HarnessError> {
    let mut claims: BTreeMap<OperationKey, Claim> = BTreeMap::new();
    for call in calls {
        let key = claim_key(call, inventory)?;
        claims
            .entry(key)
            .or_default()
            .callsites
            .insert(format!("{}:{}", call.path, call.function));
    }
    Ok(claims)
}

/// Join current empty SDK sends onto the pinned inventory rows.
pub(crate) fn scan(root: &Path, inventory: &Inventory) -> Result<Scan, HarnessError> {
    let (sources, calls, nonempty) = source_inventory(root)?;
    let closure = closure_text(&sources);
    let claims = collect_claims(&calls, inventory)?;
    Ok(Scan {
        claims,
        files: sources.len(),
        callsites: calls.len(),
        nonempty,
        sha256: sha256_hex(closure.as_bytes()),
    })
}

/// A sent operation the emulator rejected as unsupported (HTTP 501) is still
/// invoked; a decode failure is not.
pub(crate) fn exercised_status(status: &str) -> bool {
    matches!(
        status,
        "collector_success" | "explicit_sdk_probe_success" | "emulator_not_implemented"
    )
}
