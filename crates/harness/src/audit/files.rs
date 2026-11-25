use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::files::read_bytes;
use crate::provenance::sha256_hex;

pub const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;

pub const MAX_DOCUMENT_BYTES: usize = 32 * 1024 * 1024;

pub fn read_json_audit(path: &Path) -> Result<(Value, String), HarnessError> {
    let bytes = read_bytes(path, MAX_INPUT_BYTES)?;
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

pub fn read_json_document(path: &Path) -> Result<(Value, String), HarnessError> {
    let bytes = read_bytes(path, MAX_DOCUMENT_BYTES).map_err(budget_message)?;
    let digest = sha256_hex(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

pub fn read_toml_audit(path: &Path) -> Result<(Value, String), HarnessError> {
    crate::catalog::toml_doc::read(path, MAX_INPUT_BYTES)
}

pub fn read_toml_document(path: &Path) -> Result<(Value, String), HarnessError> {
    crate::catalog::toml_doc::read(path, MAX_DOCUMENT_BYTES).map_err(budget_message)
}

pub(crate) fn budget_message(error: HarnessError) -> HarnessError {
    match error {
        HarnessError::Invalid(reason) if reason.contains("input byte budget exceeded") => {
            HarnessError::invalid("coverage evidence document exceeds byte budget")
        }
        other => other,
    }
}

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

pub fn relative_slash(root: &Path, path: &Path) -> Result<String, HarnessError> {
    let resolved = fs::canonicalize(path)?;
    let base = fs::canonicalize(root)?;
    let relative = resolved
        .strip_prefix(&base)
        .map_err(|_| HarnessError::invalid("path is outside the workspace root"))?;
    Ok(slash_path(relative))
}

pub fn python_str(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::String(text) => text.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub(crate) fn value_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}

pub(crate) fn text_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, HarnessError> {
    value_field(value, key)?
        .as_str()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

pub(crate) fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, HarnessError> {
    value_field(value, key)?
        .as_array()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}
