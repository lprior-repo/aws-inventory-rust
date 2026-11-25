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

/// Encodes a JSON object in its committed TOML form.
///
/// TOML has no null, so an object member that is null is omitted; readers treat an
/// absent key as null. A null array element has no encoding and is rejected.
///
/// Numbers are converted explicitly to TOML integers or floats. Serialising a
/// `serde_json::Number` directly is not portable: with `arbitrary_precision`, which
/// feature unification enables across the workspace, it serialises as a private map.
pub fn render(value: &Value) -> Result<String, HarnessError> {
    let table = match to_toml(value, "")? {
        toml::Value::Table(table) => table,
        _ => {
            return Err(HarnessError::invalid(
                "TOML document root must be an object",
            ));
        }
    };
    toml::to_string_pretty(&table)
        .map_err(|error| HarnessError::invalid(format!("TOML encoding failed: {error}")))
}

fn to_toml(value: &Value, path: &str) -> Result<toml::Value, HarnessError> {
    Ok(match value {
        Value::Null => {
            return Err(HarnessError::invalid(format!(
                "null array element at {path} has no TOML encoding"
            )));
        }
        Value::Bool(value) => toml::Value::Boolean(*value),
        Value::Number(number) => number_to_toml(number, path)?,
        Value::String(text) => toml::Value::String(text.clone()),
        Value::Array(items) => toml::Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| to_toml(item, &format!("{path}[{index}]")))
                .collect::<Result<_, _>>()?,
        ),
        Value::Object(map) => {
            let mut out = toml::Table::new();
            for (key, item) in map {
                if !item.is_null() {
                    out.insert(key.clone(), to_toml(item, &format!("{path}/{key}"))?);
                }
            }
            toml::Value::Table(out)
        }
    })
}

fn number_to_toml(number: &serde_json::Number, path: &str) -> Result<toml::Value, HarnessError> {
    if let Some(value) = number.as_i64() {
        return Ok(toml::Value::Integer(value));
    }
    if let Some(value) = number.as_u64() {
        let value = i64::try_from(value).map_err(|_| {
            HarnessError::invalid(format!("integer at {path} exceeds TOML's i64 range"))
        })?;
        return Ok(toml::Value::Integer(value));
    }
    number
        .as_f64()
        .map(toml::Value::Float)
        .ok_or_else(|| HarnessError::invalid(format!("number at {path} has no TOML encoding")))
}
