use std::collections::BTreeSet;

use inventory_core::{Resource, validate_identifier};
use serde_json::Value;

pub(super) const MIN_IDENTIFIER_BYTES: usize = 8;

const ARN_PREFIX: &str = "arn:";

pub(super) const MIN_ARN_COMPONENT_BYTES: usize = 3;

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
    if component.is_empty() {
        return None;
    }
    Some((account_id, region, component))
}

fn primary_identifier(properties: &Value) -> Option<&str> {
    properties
        .get("resource_identifier")
        .and_then(Value::as_str)
        .filter(|identifier| identifier.len() >= MIN_IDENTIFIER_BYTES)
}

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
        .filter(|value| value.starts_with(ARN_PREFIX))
        .copied()
        .collect();
    let mut result = values;
    for value in arn_values {
        if let Some((_, _, component)) = arn_component(value)
            && component.len() >= MIN_ARN_COMPONENT_BYTES
        {
            result.insert(component);
        }
    }
    result
}

pub(super) fn is_arn(value: &str) -> bool {
    value.starts_with(ARN_PREFIX)
        && value.len() >= 11
        && value[4..].contains(':')
        && value
            .splitn(6, ':')
            .enumerate()
            .filter(|(_, part)| !part.is_empty())
            .count()
            >= 5
}

pub(super) fn arn_service_type(value: &str) -> Option<(&str, &str)> {
    let mut parts = value.splitn(6, ':');
    if parts.next() != Some("arn") {
        return None;
    }
    parts.next()?;
    let service = parts.next()?;
    if service.is_empty() {
        return None;
    }
    parts.next()?;
    parts.next()?;
    let resource = parts.next()?;
    if resource.is_empty() {
        return None;
    }
    let resource_type = resource.split('/').next()?.split(':').next()?;
    if resource_type.is_empty() {
        return None;
    }
    Some((service, resource_type))
}

pub(super) fn resource_service_type(resource: &Resource) -> (Option<String>, Option<String>) {
    if is_arn(&resource.id) {
        return match arn_service_type(&resource.id) {
            Some((s, t)) => (Some(s.to_owned()), Some(t.to_owned())),
            None => (None, None),
        };
    }
    let identifier = resource
        .properties
        .get("resource_identifier")
        .and_then(Value::as_str);
    if let Some(rid) = identifier
        && is_arn(rid)
    {
        return match arn_service_type(rid) {
            Some((s, t)) => (Some(s.to_owned()), Some(t.to_owned())),
            None => (None, None),
        };
    }
    (None, None)
}
