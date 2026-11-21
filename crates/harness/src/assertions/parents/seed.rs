//! Parent seed fixture loading and validation.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;
use crate::provenance;

pub fn fixture_path(root: &Path) -> PathBuf {
    root.join("crates/aws/tests/fixtures/cloudcontrol-parents.json")
}

pub fn load_fixtures(root: &Path) -> Result<(Value, String), HarnessError> {
    let path = fixture_path(root);
    let (document, sha) = crate::snapshot::read_json(&path)?;

    validate_fixtures(&document)?;

    Ok((document, sha))
}

fn validate_fixtures(document: &Value) -> Result<(), HarnessError> {
    if document.get("schema_version").and_then(Value::as_i64) != Some(1) {
        return Err(HarnessError::invalid("fixture schema version"));
    }
    if document.get("account_id").and_then(Value::as_str) != Some("000000000000") {
        return Err(HarnessError::invalid("fixture account ID"));
    }
    if document.get("region").and_then(Value::as_str) != Some("us-east-1") {
        return Err(HarnessError::invalid("fixture region"));
    }
    Ok(())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    provenance::sha256_hex(bytes)
}
