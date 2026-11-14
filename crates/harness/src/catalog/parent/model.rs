use std::collections::BTreeSet;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive::{Archive, Entry};
use crate::catalog::json::Json;
use crate::catalog::schema::pointer;

pub(super) fn schema_member<'a>(
    archive: &'a Archive,
    family: &str,
    resource: &str,
) -> Result<&'a Entry, HarnessError> {
    let file = format!(
        "aws-{}-{}.json",
        family.to_lowercase(),
        resource.to_lowercase()
    );
    let entry = archive
        .by_file(&file)
        .ok_or_else(|| HarnessError::invalid(format!("schema archive lacks {file}")))?;
    let expected = format!("AWS::{family}::{resource}");
    if type_name(&entry.schema) != expected {
        return Err(HarnessError::invalid(
            "schema archive filename/type mismatch",
        ));
    }
    Ok(entry)
}

pub(super) fn type_name(schema: &Value) -> &str {
    schema.get("typeName").and_then(Value::as_str).unwrap_or("")
}

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
            pointer(text)
        })
        .collect()
}

pub(super) fn sorted_pointers(schema: &Value, field: &str) -> Result<Vec<String>, HarnessError> {
    let values: BTreeSet<String> = match schema.get(field).and_then(Value::as_array) {
        Some(array) => array
            .iter()
            .map(|value| {
                let text = value
                    .as_str()
                    .ok_or_else(|| HarnessError::invalid(format!("non-string {field} pointer")))?;
                pointer(text)
            })
            .collect::<Result<BTreeSet<_>, HarnessError>>()?,
        None => BTreeSet::new(),
    };
    Ok(values.into_iter().collect())
}

pub(super) fn required_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn strings(values: &[String]) -> Json {
    Json::array(
        values
            .iter()
            .map(|value| Json::string(value.as_str()))
            .collect(),
    )
}
