use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::files::read_bytes;
use crate::provenance::sha256_hex;

/// Decodes a committed TOML catalog into the JSON-shaped value the harness reads.
pub(crate) fn parse(text: &str) -> Result<Value, HarnessError> {
    toml::from_str(text).map_err(|error| HarnessError::invalid(format!("invalid TOML: {error}")))
}

/// Reads a committed TOML catalog, returning its value and the SHA-256 of its bytes.
pub(crate) fn read(path: &Path, limit: usize) -> Result<(Value, String), HarnessError> {
    let bytes = read_bytes(path, limit)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| HarnessError::invalid(format!("{} is not UTF-8", path.display())))?;
    Ok((parse(text)?, sha256_hex(&bytes)))
}

/// Encodes a value in its committed TOML form.
///
/// TOML has no null, so an object member that is null is omitted; readers treat an
/// absent key as null. A null array element has no encoding and is rejected.
pub(crate) fn render(value: &Value) -> Result<String, HarnessError> {
    let stripped = strip_nulls(value, "")?;
    toml::to_string_pretty(&stripped)
        .map_err(|error| HarnessError::invalid(format!("TOML encoding failed: {error}")))
}

fn strip_nulls(value: &Value, path: &str) -> Result<Value, HarnessError> {
    Ok(match value {
        Value::Null => {
            return Err(HarnessError::invalid(format!(
                "null array element at {path} has no TOML encoding"
            )));
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, item) in map {
                if !item.is_null() {
                    out.insert(key.clone(), strip_nulls(item, &format!("{path}/{key}"))?);
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| strip_nulls(item, &format!("{path}[{index}]")))
                .collect::<Result<_, _>>()?,
        ),
        other => other.clone(),
    })
}
