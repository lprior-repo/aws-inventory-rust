use inventory_core::{Resource, validate_identifier};
use serde_json::Value;
use std::collections::BTreeSet;

const MIN_IDENTIFIER_BYTES: usize = 8;

pub(super) const MIN_ARN_COMPONENT_BYTES: usize = 3;

const PRIMARY_IDENTIFIER: &str = "resource_identifier";

pub(super) fn identity_strings(resource: &Resource) -> BTreeSet<&str> {
    let mut values: BTreeSet<&str> = BTreeSet::new();
    if !resource.id.is_empty() && validate_identifier(&resource.id, "identity").is_ok() {
        values.insert(resource.id.as_str());
    }
    if let Some(identifier) = primary_identifier(&resource.properties)
        && identifier.len() >= MIN_IDENTIFIER_BYTES
    {
        values.insert(identifier);
    }
    let arn_values: Vec<&str> = values
        .iter()
        .copied()
        .filter(|value| is_arn(value))
        .collect();
    for value in arn_values {
        if let Some(component) = trailing_identifier(value)
            && component.len() >= MIN_ARN_COMPONENT_BYTES
        {
            values.insert(component);
        }
    }
    values
}

fn primary_identifier(properties: &Value) -> Option<&str> {
    properties
        .get(PRIMARY_IDENTIFIER)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

fn trailing_identifier(id: &str) -> Option<&str> {
    if !is_arn(id) {
        return None;
    }
    let resource = id.splitn(6, ':').nth(5)?;
    let after_slash = resource
        .rsplit_once('/')
        .map_or(resource, |(_, trailing)| trailing);
    let trailing = after_slash
        .rsplit_once(':')
        .map_or(after_slash, |(_, trailing)| trailing);
    (!trailing.is_empty()).then_some(trailing)
}

pub(super) fn arn_component(value: &str) -> Option<(&str, &str, &str)> {
    let mut parts = value.splitn(6, ':');
    if parts.next() != Some("arn") {
        return None;
    }
    parts.next()?;
    parts.next()?;
    let region = parts.next()?;
    let account_id = parts.next()?;
    let resource = parts.next()?;
    if resource.is_empty() {
        return None;
    }
    let after_slash = resource
        .rsplit_once('/')
        .map_or(resource, |(_, component)| component);
    let component = after_slash
        .rsplit_once(':')
        .map_or(after_slash, |(_, component)| component);
    (!component.is_empty()).then_some((account_id, region, component))
}

pub(super) fn is_arn(value: &str) -> bool {
    let mut parts = value.splitn(6, ':');
    parts.next() == Some("arn")
        && parts.next().is_some_and(|part| !part.is_empty())
        && parts.next().is_some_and(|part| !part.is_empty())
        && parts.next().is_some()
        && parts.next().is_some()
        && parts.next().is_some_and(|part| !part.is_empty())
}
