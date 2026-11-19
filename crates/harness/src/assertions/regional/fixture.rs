//! Regional resource fixture loading and validation.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::provenance;
use crate::HarnessError;

pub fn fixture_path(root: &Path) -> PathBuf {
    root.join("crates/aws/tests/fixtures/cloudcontrol-regional.json")
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
    if document
        .get("region")
        .and_then(Value::as_str)
        != Some("us-east-1")
    {
        return Err(HarnessError::invalid("fixture region"));
    }
    Ok(())
}

pub fn validate(fixtures: &Value, checks: &mut Vec<Value>) -> bool {
    let iam = fixtures.get("iam");
    if iam.is_none() {
        checks.push(serde_json::json!({
            "check": "regional:iam_fixture_present",
            "passed": false,
            "detail": "missing iam fixture",
        }));
        return false;
    }
    true
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    provenance::sha256_hex(bytes)
}