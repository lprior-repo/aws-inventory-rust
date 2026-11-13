//! Property paths, redaction and columns for one descriptor row.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::cloudcontrol::{optional_pointers, pointers, properties, sorted_pointers};
use crate::catalog::json::Json;
use crate::catalog::rules::{
    RESERVED, SELF_ARNS, escaped_pointer, sensitive_paths, sensitive_paths_for,
};
use crate::catalog::schema::{basic_type, snake, valid_column};

use super::MAX_COLUMNS;

pub(super) fn sensitive_set(
    name: &str,
    properties: &BTreeMap<String, Value>,
    global: &BTreeSet<String>,
    is_new: bool,
    schema: &Value,
) -> Result<Vec<String>, HarnessError> {
    let mut sensitive: BTreeSet<String> = sensitive_paths(name)
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    for key in properties.keys() {
        if global.contains(&snake(key).replace('_', "")) {
            sensitive.insert(format!("/{key}"));
        }
    }
    if is_new {
        sensitive.extend(sensitive_paths_for(name, schema)?);
    }
    Ok(sensitive.into_iter().collect())
}

pub(super) struct Paths {
    pub(super) properties: BTreeMap<String, Value>,
    pub(super) primary: Vec<String>,
    pub(super) write_only: Vec<String>,
    pub(super) sensitive: Vec<String>,
    pub(super) columns: Json,
}

pub(super) fn paths(
    name: &str,
    schema: &Value,
    global: &BTreeSet<String>,
    is_new: bool,
) -> Result<Paths, HarnessError> {
    let properties = properties(schema);
    let primary = pointers(schema, "primaryIdentifier")?;
    let write_only = sorted_pointers(schema, "writeOnlyProperties")?;
    let sensitive = sensitive_set(name, &properties, global, is_new, schema)?;
    let redacted = redacted_paths(&write_only, &sensitive);
    let columns = columns(name, &properties, schema, &redacted)?;
    Ok(Paths {
        properties,
        primary,
        write_only,
        sensitive,
        columns,
    })
}

pub(super) fn redacted_paths(write_only: &[String], sensitive: &[String]) -> BTreeSet<String> {
    let mut redacted: BTreeSet<String> = BTreeSet::new();
    for value in write_only.iter().chain(sensitive.iter()) {
        if value.matches('/').count() == 1 {
            redacted.insert(value.trim_start_matches('/').to_string());
        }
    }
    redacted
}

pub(super) fn arn_paths(
    name: &str,
    resource: &str,
    paths: &Paths,
    is_new: bool,
    schema: &Value,
) -> Result<BTreeSet<String>, HarnessError> {
    let mut arns = base_arns(&paths.properties, &paths.primary);
    if name == "AWS::SecretsManager::Secret" {
        arns.insert("/Id".to_string());
    }
    if let Some(value) = SELF_ARNS.iter().find(|(key, _)| *key == name) {
        arns.insert((value.1).to_string());
    }
    if is_new {
        arns.extend(new_arns(resource, &paths.properties, schema)?);
    }
    Ok(arns)
}

fn base_arns(properties: &BTreeMap<String, Value>, primary: &[String]) -> BTreeSet<String> {
    let mut arns: BTreeSet<String> = BTreeSet::new();
    for key in ["Arn", "ARN"] {
        if properties.contains_key(key) {
            arns.insert(format!("/{key}"));
        }
    }
    for value in primary {
        if value.to_lowercase().ends_with("arn") {
            arns.insert(value.clone());
        }
    }
    arns
}

fn new_arns(
    resource: &str,
    properties: &BTreeMap<String, Value>,
    schema: &Value,
) -> Result<BTreeSet<String>, HarnessError> {
    let readonly = optional_pointers(schema, "readOnlyProperties")?;
    let mut arns = BTreeSet::new();
    for key in [
        format!("{resource}Arn"),
        format!("{resource}ARN"),
        "ResourceArn".to_string(),
        "ResourceARN".to_string(),
    ] {
        if properties.contains_key(&key) && readonly.contains(&format!("/{key}")) {
            arns.insert(format!("/{key}"));
        }
    }
    Ok(arns)
}

pub(super) fn columns(
    name: &str,
    properties: &BTreeMap<String, Value>,
    schema: &Value,
    redacted: &BTreeSet<String>,
) -> Result<Json, HarnessError> {
    let mut output = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (key, prop) in properties {
        if redacted.contains(key) {
            continue;
        }
        output.push(column(name, key, prop, schema, &mut names)?);
    }
    if output.len() > MAX_COLUMNS {
        return Err(HarnessError::invalid(format!("too many columns: {name}")));
    }
    Ok(Json::array(output))
}

fn column(
    name: &str,
    key: &str,
    prop: &Value,
    schema: &Value,
    names: &mut BTreeSet<String>,
) -> Result<Json, HarnessError> {
    let column = unique_column(name, key, names)?;
    Ok(Json::object(vec![
        ("name", Json::string(column)),
        ("pointer", Json::string(escaped_pointer(key))),
        ("json_type", Json::string(basic_type(prop, schema)?)),
    ]))
}

fn unique_column(
    name: &str,
    key: &str,
    names: &mut BTreeSet<String>,
) -> Result<String, HarnessError> {
    let mut column = snake(key);
    if RESERVED.contains(&column.as_str()) {
        column = format!("provider_{column}");
    }
    if RESERVED.contains(&column.as_str()) {
        column = format!("configuration_{}", snake(key));
    }
    if RESERVED.contains(&column.as_str()) || names.contains(&column) || !valid_column(&column) {
        return Err(HarnessError::invalid(format!(
            "invalid/duplicate column {name}:{column}"
        )));
    }
    names.insert(column.clone());
    Ok(column)
}

pub(super) fn name_pointers(
    resource: &str,
    properties: &BTreeMap<String, Value>,
    primary: &[String],
) -> Vec<String> {
    let mut output = Vec::new();
    let resource_name = format!("{resource}Name");
    for key in ["Name", resource_name.as_str()] {
        if properties.contains_key(key) {
            output.push(format!("/{key}"));
        }
    }
    for value in primary {
        if value.ends_with("Name") && !output.iter().any(|known| known == value) {
            output.push(value.clone());
        }
    }
    output
}
