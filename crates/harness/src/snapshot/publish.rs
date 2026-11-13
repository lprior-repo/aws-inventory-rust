//! Snapshot publication and its operator receipt.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde_json::{Value, json};

use super::MAX_DOCUMENT_BYTES;
use super::binding::Baseline;
use crate::HarnessError;
use crate::provenance::sha256_hex;

/// Assemble the frozen document from the validated baseline.
pub(super) fn snapshot_document(
    summary_sha: &str,
    role: &str,
    emulator: &Value,
    baseline: &Baseline,
    cases: Vec<Value>,
) -> Value {
    json!({
        "schema_version": 1,
        "account_id": "000000000000",
        "region": "us-east-1",
        "assume_role_arn": role,
        "provenance": provenance(summary_sha, emulator, baseline),
        "cases": cases,
    })
}

fn provenance(summary_sha: &str, emulator: &Value, baseline: &Baseline) -> Value {
    let commit = emulator
        .get("source_commit")
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "emulator_version": "1.0.336",
        "emulator_commit": commit,
        "summary_sha256": summary_sha,
        "baseline_binary_sha256": baseline.binary_sha,
        "normalization": "remove only the top-level observed_at field from resource and edge observations",
    })
}

/// Write the oracle exactly once; an existing destination is never clobbered.
pub(super) fn publish(destination: &Path, document: &Value) -> Result<(), HarnessError> {
    let mut encoded = serde_json::to_string_pretty(document)?;
    encoded.push('\n');
    if encoded.len() > MAX_DOCUMENT_BYTES {
        return Err(HarnessError::invalid(
            "snapshot output byte budget exceeded",
        ));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    file.write_all(encoded.as_bytes())?;
    file.flush()?;
    Ok(())
}

/// Describe the published oracle and its case cardinalities.
pub(super) fn receipt(
    evidence: &Path,
    destination: &Path,
    summary_sha: &str,
    document: &Value,
) -> Result<Value, HarnessError> {
    let snapshot = std::fs::canonicalize(destination)?;
    let bytes = std::fs::read(&snapshot)?;
    let summary = std::fs::canonicalize(evidence.join("summary.json"))?;
    let cases = document
        .get("cases")
        .and_then(Value::as_array)
        .map(|cases| cases.iter().map(case_receipt).collect::<Vec<_>>())
        .unwrap_or_default();
    Ok(json!({
        "snapshot": snapshot.to_string_lossy(),
        "snapshot_sha256": sha256_hex(&bytes),
        "summary": summary.to_string_lossy(),
        "summary_sha256": summary_sha,
        "cases": cases,
    }))
}

fn case_receipt(case: &Value) -> Value {
    json!({
        "name": case.get("name").cloned().unwrap_or(Value::Null),
        "status": case.get("status").cloned().unwrap_or(Value::Null),
        "resources": length(case, "resources"),
        "edges": length(case, "edges"),
        "scopes": length(case, "scopes"),
        "failures": length(case, "failures"),
    })
}

fn length(case: &Value, key: &str) -> usize {
    case.get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}
