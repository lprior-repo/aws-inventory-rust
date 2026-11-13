//! Freeze independently asserted emulator observations for the Rust tests.
//!
//! Only the collector's own observation timestamp is removed. Provider
//! timestamps, properties, duplicate observations, edges, page boundaries and
//! failure details stay inside the oracle. A newly seeded process needs a new
//! snapshot: provenance chaining refuses a baseline from any other tree.

mod binding;
mod cases;
mod db;
mod publish;
mod summary;

use std::path::Path;

use serde_json::{Map, Value};

use crate::HarnessError;
use crate::provenance::sha256_hex;

pub use binding::{Baseline, execution_binding};
use db::collect_cases;
use publish::{publish, receipt, snapshot_document};

/// Maximum bytes accepted for any intermediate JSON document.
pub const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
/// Maximum rows read from any single observation or scope query.
pub const MAX_ROWS: usize = 100_000;
/// The five baseline cases and their command receipt names.
pub const CASES: [(&str, &str); 5] = [
    ("native", "native"),
    ("sources", "sources"),
    ("cloudcontrol", "cloud-control"),
    ("assumedrole", "assumed-role"),
    ("parents", "parents"),
];

/// Read one bounded JSON document and its raw-byte digest.
pub fn read_json(path: &Path) -> Result<(Value, String), HarnessError> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(HarnessError::invalid(format!(
            "JSON document exceeds its byte budget: {}",
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        )));
    }
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

pub(crate) fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}

pub(crate) fn object(value: &Value) -> Result<&Map<String, Value>, HarnessError> {
    value
        .as_object()
        .ok_or_else(|| HarnessError::invalid("expected a JSON object"))
}

/// Freeze the oracle and return the receipt describing it.
pub async fn freeze(
    evidence: &Path,
    destination: &Path,
    binary: &Path,
    root: &Path,
) -> Result<Value, HarnessError> {
    let evidence = std::fs::canonicalize(evidence)?;
    let (summary, summary_sha) = read_json(&evidence.join("summary.json"))?;
    summary::validate(&summary)?;
    let baseline = execution_binding(&summary, &evidence, binary, root)?;
    let emulator = summary.get("emulator").cloned().unwrap_or(Value::Null);
    let role = summary::assume_role(&evidence)?;
    let arguments = cases::case_arguments(&evidence, &role, &baseline.catalog, &baseline.parents)?;
    let cases = collect_cases(&evidence, &baseline, &arguments).await?;
    // Recheck every input after the stable read transaction; never publish an
    // oracle across a concurrent source, catalog or binary edit.
    execution_binding(&summary, &evidence, binary, root)?;
    let document = snapshot_document(&summary_sha, &role, &emulator, &baseline, cases);
    publish(destination, &document)?;
    receipt(&evidence, destination, &summary_sha, &document)
}
