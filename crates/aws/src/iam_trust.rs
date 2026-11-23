use inventory_core::validate_identifier;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use thiserror::Error;

const MAX_DOCUMENT_BYTES: usize = 65_536;
const MAX_STATEMENTS: usize = 128;
const MAX_PRINCIPALS: usize = 256;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TrustSummaryError {
    #[error("trust policy exceeds 64 KiB")]
    DocumentTooLarge,
    #[error("trust policy is not UTF-8")]
    Utf8,
    #[error("trust policy is not valid JSON")]
    Json,
    #[error("trust policy lacks Statement")]
    MissingStatement,
    #[error("trust policy statements exceed bounds or have invalid shape")]
    Statements,
    #[error("trust policy exceeds 256 principals")]
    Principals,
    #[error("trust reference exceeds bounds or has invalid shape")]
    Reference,
    #[error("trust statement is not an object")]
    Statement,
    #[error("invalid trust principal shape")]
    Principal,
    #[error("trust condition operators exceed bounds")]
    Conditions,
    #[error(transparent)]
    Validation(#[from] inventory_core::ValidationError),
}

pub fn summarize(document: Option<&str>) -> Result<Vec<Value>, TrustSummaryError> {
    let Some(document) = document else {
        return Ok(Vec::new());
    };
    let policy = decode(document)?;
    let statements = policy
        .get("Statement")
        .ok_or(TrustSummaryError::MissingStatement)?;
    let summaries = match statements {
        Value::Array(values) if values.len() <= MAX_STATEMENTS => values
            .iter()
            .map(summarize_statement)
            .collect::<Result<Vec<_>, _>>()?,
        Value::Object(_) => vec![summarize_statement(statements)?],
        _ => return Err(TrustSummaryError::Statements),
    };
    check_principal_count(&summaries)?;
    Ok(summaries)
}

fn decode(document: &str) -> Result<Value, TrustSummaryError> {
    if document.len() > MAX_DOCUMENT_BYTES {
        return Err(TrustSummaryError::DocumentTooLarge);
    }
    let decoded = percent_encoding::percent_decode_str(document)
        .decode_utf8()
        .map_err(|_| TrustSummaryError::Utf8)?;
    serde_json::from_str(&decoded).map_err(|_| TrustSummaryError::Json)
}

fn check_principal_count(summaries: &[Value]) -> Result<(), TrustSummaryError> {
    summaries
        .iter()
        .filter_map(|summary| summary.get("principals").and_then(Value::as_object))
        .flat_map(|principals| principals.values())
        .filter_map(Value::as_array)
        .try_fold(0_usize, |count, values| {
            count
                .checked_add(values.len())
                .filter(|count| *count <= MAX_PRINCIPALS)
                .ok_or(TrustSummaryError::Principals)
        })?;
    Ok(())
}

fn checked_text(value: &str) -> Result<String, TrustSummaryError> {
    validate_identifier(value, "trust reference")?;
    Ok(value.to_owned())
}

fn string_values(value: Option<&Value>, limit: usize) -> Result<Vec<String>, TrustSummaryError> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::String(value)) => Ok(vec![checked_text(value)?]),
        Some(Value::Array(values)) if values.len() <= limit => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or(TrustSummaryError::Reference)
                    .and_then(checked_text)
            })
            .collect(),
        _ => Err(TrustSummaryError::Reference),
    }
}

fn principals(value: Option<&Value>) -> Result<BTreeMap<&str, Vec<String>>, TrustSummaryError> {
    match value {
        Some(Value::String(value)) if value == "*" => {
            Ok(BTreeMap::from([("AWS", vec![value.clone()])]))
        }
        Some(Value::Object(values)) => ["AWS", "Service", "Federated", "CanonicalUser"]
            .into_iter()
            .try_fold(BTreeMap::new(), |mut result, kind| {
                let references = string_values(values.get(kind), MAX_PRINCIPALS)?;
                if !references.is_empty() {
                    result.insert(kind, references);
                }
                Ok(result)
            }),
        None => Ok(BTreeMap::new()),
        _ => Err(TrustSummaryError::Principal),
    }
}

fn summarize_statement(statement: &Value) -> Result<Value, TrustSummaryError> {
    let object = statement.as_object().ok_or(TrustSummaryError::Statement)?;
    let condition_keys: Vec<&str> = object
        .get("Condition")
        .and_then(Value::as_object)
        .map_or_else(Vec::new, |conditions| {
            conditions.keys().map(String::as_str).collect()
        });
    if condition_keys.len() > 4_096 {
        return Err(TrustSummaryError::Conditions);
    }
    Ok(json!({
        "effect": object.get("Effect").and_then(Value::as_str),
        "actions": string_values(object.get("Action"), 64)?,
        "principals": principals(object.get("Principal"))?,
        "has_not_principal": object.contains_key("NotPrincipal"),
        "has_not_action": object.contains_key("NotAction"),
        "condition_operators": condition_keys,
        "condition_values_omitted": object.contains_key("Condition"),
    }))
}

#[cfg(test)]
mod tests;
