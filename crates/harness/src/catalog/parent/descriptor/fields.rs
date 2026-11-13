//! Sensitive paths, columns and relationships for one parent descriptor row.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::json::Json;
use crate::catalog::rules::{RESERVED, Rule, sensitive_paths_for};
use crate::catalog::schema::{basic_type, snake, valid_column};
use crate::catalog::tables::{NATIVE_KINDS, RELATIONSHIPS, SENSITIVE};

pub(super) fn sensitive_set(
    name: &str,
    properties: &BTreeMap<String, Value>,
    global: &BTreeSet<String>,
    schema: &Value,
) -> Result<Vec<String>, HarnessError> {
    let mut sensitive: BTreeSet<String> = SENSITIVE
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, values)| values.iter().map(|value| (*value).to_string()).collect())
        .unwrap_or_default();
    sensitive.extend(sensitive_paths_for(name, schema)?);
    for key in properties.keys() {
        if global.contains(&snake(key).replace('_', "")) {
            sensitive.insert(format!("/{key}"));
        }
    }
    Ok(sensitive.into_iter().collect())
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

pub(super) fn columns(
    name: &str,
    properties: &BTreeMap<String, Value>,
    schema: &Value,
    redacted: &BTreeSet<String>,
) -> Result<Vec<Json>, HarnessError> {
    let mut columns = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (key, prop) in properties {
        if redacted.contains(key) {
            continue;
        }
        columns.push(column(name, key, prop, schema, &mut names)?);
    }
    Ok(columns)
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
        ("pointer", Json::string(format!("/{key}"))),
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
    if RESERVED.contains(&column.as_str()) || names.contains(&column) || !valid_column(&column) {
        return Err(HarnessError::invalid(format!(
            "invalid/duplicate provider column {name}:{column}"
        )));
    }
    names.insert(column.clone());
    Ok(column)
}

pub(super) fn arn_paths(
    resource: &str,
    properties: &BTreeMap<String, Value>,
    primary: &[String],
) -> BTreeSet<String> {
    let mut arns: BTreeSet<String> = BTreeSet::new();
    let resource_arn = format!("{resource}Arn");
    for key in ["Arn", "ARN", resource_arn.as_str()] {
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

pub(super) fn native_kind(family: &str, resource: &str, name: &str) -> String {
    NATIVE_KINDS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| (*value).to_string())
        .unwrap_or_else(|| format!("aws_{}_{}", snake(family), snake(resource)))
}

pub(super) fn name_pointers(resource: &str, properties: &BTreeMap<String, Value>) -> Vec<String> {
    let mut name_pointers = Vec::new();
    let resource_name = format!("{resource}Name");
    for key in ["Name", resource_name.as_str()] {
        if properties.contains_key(key) {
            name_pointers.push(format!("/{key}"));
        }
    }
    name_pointers
}

pub(super) fn seeds(name: &str) -> Vec<Rule> {
    RELATIONSHIPS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, rules)| {
            rules
                .iter()
                .map(|(path, relation, target, arn)| Rule::reviewed(path, relation, target, *arn))
                .collect()
        })
        .unwrap_or_default()
}
