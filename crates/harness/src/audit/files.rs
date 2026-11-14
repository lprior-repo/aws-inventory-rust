//! Bounded byte, digest and path helpers shared by both audit commands.

use std::fs;
use std::path::{Component, Path, PathBuf};

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

/// Resolve a possibly nonexistent output path without requiring the file.
///
/// Existing components are canonicalized; the remainder is normalised
/// lexically, matching `Path.resolve()` on a path that is about to be created.
pub fn resolve_target(path: &Path) -> Result<PathBuf, HarnessError> {
    if let Ok(resolved) = fs::canonicalize(path) {
        return Ok(resolved);
    }
    let absolute = absolute_path(path)?;
    Ok(canonical_ancestor(&lexical_normalise(&absolute)))
}

/// Join a relative path onto the working directory.
fn absolute_path(path: &Path) -> Result<PathBuf, HarnessError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

/// Drop `.` components and resolve `..` lexically.
fn lexical_normalise(absolute: &Path) -> PathBuf {
    let mut normalised = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                normalised.pop();
            }
            Component::CurDir => {}
            other => normalised.push(other.as_os_str()),
        }
    }
    normalised
}

/// Canonicalize the longest existing prefix and re-append the tail.
fn canonical_ancestor(normalised: &Path) -> PathBuf {
    let mut existing = normalised;
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    loop {
        if let Ok(base) = fs::canonicalize(existing) {
            let mut resolved = base;
            for part in tail.iter().rev() {
                resolved.push(part);
            }
            return resolved;
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name.to_os_string());
                existing = parent;
            }
            _ => return normalised.to_path_buf(),
        }
    }
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

/// Decode a source file, refusing invalid UTF-8 the way Python's `decode` does.
pub fn decode_utf8(bytes: Vec<u8>, path: &Path) -> Result<String, HarnessError> {
    String::from_utf8(bytes)
        .map_err(|_| HarnessError::invalid(format!("{}: source is not UTF-8", path.display())))
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

/// Borrow an object member.
pub(crate) fn object_field<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>, HarnessError> {
    value_field(value, key)?
        .as_object()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}
