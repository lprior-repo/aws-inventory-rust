//! Reproduction of the reviewed 397-type Cloud Control root catalog.

mod build;
mod descriptor;

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::json::Json;

pub use build::build_resources;

/// The reviewed type count the generator refuses to drift from.
pub const EXPECTED_TYPES: usize = 397;

/// The archive base name embedded in evidence documents.
pub fn archive_name() -> &'static str {
    crate::catalog::ARCHIVE
        .rsplit('/')
        .next()
        .unwrap_or(crate::catalog::ARCHIVE)
}

/// Sorted property map for one schema.
pub(super) fn properties(schema: &Value) -> BTreeMap<String, Value> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// One handler's declared sub-schema.
pub(super) fn handler_schema<'a>(schema: &'a Value, operation: &str) -> Option<&'a Value> {
    schema.get("handlers")?.get(operation)?.get("handlerSchema")
}

/// Sorted, de-duplicated permissions declared by one handler.
pub(super) fn permissions(schema: &Value, operation: &str) -> Vec<String> {
    let mut values: BTreeSet<String> = BTreeSet::new();
    if let Some(list) = schema
        .get("handlers")
        .and_then(|handlers| handlers.get(operation))
        .and_then(|handler| handler.get("permissions"))
        .and_then(Value::as_array)
    {
        for value in list {
            if let Some(name) = value.as_str() {
                values.insert(name.to_string());
            }
        }
    }
    values.into_iter().collect()
}

/// Sorted `/properties/...` pointers from a schema array.
pub(super) fn pointers(schema: &Value, field: &str) -> Result<Vec<String>, HarnessError> {
    let values = schema
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid(format!("schema lacks {field}")))?;
    values
        .iter()
        .map(|value| {
            let text = value
                .as_str()
                .ok_or_else(|| HarnessError::invalid(format!("non-string {field} pointer")))?;
            crate::catalog::schema::pointer(text)
        })
        .collect()
}

/// Pointer set from an optional schema array.
pub(super) fn optional_pointers(
    schema: &Value,
    field: &str,
) -> Result<BTreeSet<String>, HarnessError> {
    match schema.get(field).and_then(Value::as_array) {
        Some(array) => array
            .iter()
            .map(|value| {
                let text = value
                    .as_str()
                    .ok_or_else(|| HarnessError::invalid(format!("non-string {field} pointer")))?;
                crate::catalog::schema::pointer(text)
            })
            .collect(),
        None => Ok(BTreeSet::new()),
    }
}

/// Sorted, de-duplicated pointer array; an absent field is empty.
pub(super) fn sorted_pointers(schema: &Value, field: &str) -> Result<Vec<String>, HarnessError> {
    let values: BTreeSet<String> = match schema.get(field).and_then(Value::as_array) {
        Some(array) => array
            .iter()
            .map(|value| {
                let text = value
                    .as_str()
                    .ok_or_else(|| HarnessError::invalid(format!("non-string {field} pointer")))?;
                crate::catalog::schema::pointer(text)
            })
            .collect::<Result<BTreeSet<_>, _>>()?,
        None => BTreeSet::new(),
    };
    Ok(values.into_iter().collect())
}

/// Serialize a string slice as a JSON array.
pub(super) fn strings(values: &[String]) -> Json {
    Json::array(
        values
            .iter()
            .map(|value| Json::string(value.as_str()))
            .collect(),
    )
}

/// Python truthiness for schema fragments.
pub(super) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(object) => !object.is_empty(),
    }
}
