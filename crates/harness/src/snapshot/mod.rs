mod binding;
mod cases;
mod db;
mod publish;
mod summary;

#[cfg(test)]
mod tests;

use std::path::Path;

use serde_json::{Map, Value};

use crate::HarnessError;
use crate::files::{ReadError, read_limited};
use crate::provenance::sha256_hex;

pub use binding::{Baseline, execution_binding};
use db::collect_cases;
use publish::{publish, receipt, snapshot_document};

pub const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;

pub const MAX_ROWS: usize = 100_000;

pub const CASES: [(&str, &str); 5] = [
    ("native", "native"),
    ("sources", "sources"),
    ("cloudcontrol", "cloud-control"),
    ("assumedrole", "assumed-role"),
    ("parents", "parents"),
];

pub fn read_json(path: &Path) -> Result<(Value, String), HarnessError> {
    read_json_with_limit(path, MAX_DOCUMENT_BYTES)
}

pub fn read_toml(path: &Path) -> Result<(Value, String), HarnessError> {
    crate::catalog::toml_doc::read(path, MAX_DOCUMENT_BYTES)
}

fn read_json_with_limit(path: &Path, limit: usize) -> Result<(Value, String), HarnessError> {
    let file = std::fs::File::open(path)?;
    let bytes = read_limited(file, limit).map_err(|error| document_error(path, error))?;
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

fn document_error(path: &Path, error: ReadError) -> HarnessError {
    match error {
        ReadError::Io(error) => HarnessError::Io(error),
        ReadError::TooLarge => {
            let name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => std::borrow::Cow::Borrowed(""),
            };
            HarnessError::invalid(format!("JSON document exceeds its byte budget: {name}"))
        }
        ReadError::InvalidBudget => HarnessError::invalid("input byte budget is not representable"),
    }
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

    execution_binding(&summary, &evidence, binary, root)?;
    let document = snapshot_document(&summary_sha, &role, &emulator, &baseline, cases);
    publish(destination, &document)?;
    receipt(&evidence, destination, &summary_sha, &document)
}
