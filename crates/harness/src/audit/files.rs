//! Bounded byte, digest and path helpers shared by both audit commands.

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::files::read_bytes;
use crate::provenance::sha256_hex;

/// Maximum bytes accepted for one audited input document (the Python tool).
pub const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
/// Maximum bytes accepted for one coverage evidence document.
pub const MAX_DOCUMENT_BYTES: usize = 32 * 1024 * 1024;

/// Read one bounded audit document and the digest of its raw bytes.
pub fn read_json_audit(path: &Path) -> Result<(Value, String), HarnessError> {
    let bytes = read_bytes(path, MAX_INPUT_BYTES)?;
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

/// Read one coverage evidence document with its wider byte budget.
pub fn read_json_document(path: &Path) -> Result<(Value, String), HarnessError> {
    let bytes = read_bytes(path, MAX_DOCUMENT_BYTES).map_err(budget_message)?;
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

/// Rewrite the generic byte-budget reason for coverage documents.
pub(crate) fn budget_message(error: HarnessError) -> HarnessError {
    match error {
        HarnessError::Invalid(reason) if reason.contains("input byte budget exceeded") => {
            HarnessError::invalid("coverage evidence document exceeds byte budget")
        }
        other => other,
    }
}

/// Slash-separated path relative to the workspace root.
pub fn slash_path(relative: &Path) -> String {
    let mut text = String::new();
    for component in relative.components() {
        if !text.is_empty() {
            text.push('/');
        }
        text.push_str(&component.as_os_str().to_string_lossy());
    }
    text
}

/// Resolve `path` under `root` and render it the way Python prints it.
pub fn relative_slash(root: &Path, path: &Path) -> Result<String, HarnessError> {
    let resolved = fs::canonicalize(path)?;
    let base = fs::canonicalize(root)?;
    let relative = resolved
        .strip_prefix(&base)
        .map_err(|_| HarnessError::invalid("path is outside the workspace root"))?;
    Ok(slash_path(relative))
}

/// Render a JSON scalar with Python's `str()` semantics.
pub fn python_str(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::String(text) => text.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Borrow a JSON object member with a concrete reason on absence.
pub(crate) fn value_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}

/// Borrow a string member.
pub(crate) fn text_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, HarnessError> {
    value_field(value, key)?
        .as_str()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

/// Borrow an array member.
pub(crate) fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, HarnessError> {
    value_field(value, key)?
        .as_array()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}
