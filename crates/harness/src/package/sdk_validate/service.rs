//! Per-service snapshot validation against the official upstream table.

use std::collections::BTreeSet;

use regex::Regex;
use serde_json::Value;

use crate::HarnessError;

use super::{CRATE_PREFIX, Patterns, REPOSITORY};

/// Validate one snapshot service against the official upstream table.
pub(super) fn validate_service(
    service: &Value,
    document: &Value,
    upstream: &toml::Table,
    revision: &str,
    patterns: &Patterns,
) -> Result<(), HarnessError> {
    let name = service_name(service)?;
    let version = check_version(service, name, patterns)?;
    check_feature(service, name)?;
    check_client_features(service, name, &patterns.feature)?;
    let entry = upstream.get(name).ok_or_else(|| upstream_error(name))?;
    check_upstream(service, entry, name, version)?;
    check_model(service, entry, name)?;
    check_checksum(service, name, &patterns.hex)?;
    check_origin(service, document, revision, name)
}

fn service_name(service: &Value) -> Result<&str, HarnessError> {
    service
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("invalid crate name or stable version: missing"))
}

fn check_version<'a>(
    service: &'a Value,
    name: &str,
    patterns: &Patterns,
) -> Result<&'a str, HarnessError> {
    match service.get("version").and_then(Value::as_str) {
        Some(version) if patterns.name.is_match(name) && patterns.version.is_match(version) => {
            Ok(version)
        }
        _ => Err(HarnessError::invalid(format!(
            "invalid crate name or stable version: {name}"
        ))),
    }
}

fn check_feature(service: &Value, name: &str) -> Result<(), HarnessError> {
    let expected_feature = name.strip_prefix(CRATE_PREFIX).unwrap_or(name);
    if service.get("feature").and_then(Value::as_str) != Some(expected_feature) {
        return Err(HarnessError::invalid(format!(
            "feature does not match crate name: {name}"
        )));
    }
    Ok(())
}

fn check_client_features(
    service: &Value,
    name: &str,
    feature_pattern: &Regex,
) -> Result<(), HarnessError> {
    let features = client_features(service.get("client_features"));
    let features_valid = features
        .as_ref()
        .map(|items| valid_features(items, feature_pattern))
        .unwrap_or(false);
    if !features_valid {
        return Err(HarnessError::invalid(format!(
            "invalid modern SDK client feature selection: {name}"
        )));
    }
    Ok(())
}

fn upstream_error(name: &str) -> HarnessError {
    HarnessError::invalid(format!(
        "version does not match official source or is yanked: {name}"
    ))
}

fn check_upstream(
    service: &Value,
    entry: &toml::Value,
    name: &str,
    version: &str,
) -> Result<(), HarnessError> {
    let upstream_version = entry.get("version").and_then(toml::Value::as_str);
    if Some(version) != upstream_version || truthy(service.get("yanked")) {
        return Err(upstream_error(name));
    }
    if service.get("sdk_source_sha256").and_then(Value::as_str)
        != entry.get("source_hash").and_then(toml::Value::as_str)
    {
        return Err(HarnessError::invalid(format!(
            "source fingerprint does not match official source: {name}"
        )));
    }
    Ok(())
}

fn check_model(service: &Value, entry: &toml::Value, name: &str) -> Result<(), HarnessError> {
    let model = service.get("model_sha256").and_then(Value::as_str);
    if model != entry.get("model_hash").and_then(toml::Value::as_str) {
        return Err(HarnessError::invalid(format!(
            "model fingerprint does not match official source: {name}"
        )));
    }
    Ok(())
}

fn check_checksum(service: &Value, name: &str, hex_pattern: &Regex) -> Result<(), HarnessError> {
    let checksum_valid = service
        .get("checksum_sha256")
        .and_then(Value::as_str)
        .map(|checksum| hex_pattern.is_match(checksum))
        .unwrap_or(false);
    if !checksum_valid {
        return Err(HarnessError::invalid(format!(
            "invalid registry checksum: {name}"
        )));
    }
    Ok(())
}

fn check_origin(
    service: &Value,
    document: &Value,
    revision: &str,
    name: &str,
) -> Result<(), HarnessError> {
    let revision = document
        .get("official_revision")
        .and_then(Value::as_str)
        .unwrap_or(revision);
    let expected_feature = name.strip_prefix(CRATE_PREFIX).unwrap_or(name);
    let expected_origin = format!("{REPOSITORY}/tree/{revision}/sdk/{expected_feature}");
    if service.get("origin").and_then(Value::as_str) != Some(expected_origin.as_str()) {
        return Err(HarnessError::invalid(format!(
            "unexpected source origin: {name}"
        )));
    }
    Ok(())
}

/// Python truthiness for a JSON field; a missing field is treated as present.
fn truthy(value: Option<&Value>) -> bool {
    match value {
        None => true,
        Some(Value::Null) => false,
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(number)) => number.as_f64() != Some(0.0),
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(fields)) => !fields.is_empty(),
    }
}

/// Extract the declared client feature strings, rejecting non-string entries.
fn client_features(value: Option<&Value>) -> Option<Vec<&str>> {
    let items = value?.as_array()?;
    let mut features = Vec::with_capacity(items.len());
    for item in items {
        features.push(item.as_str()?);
    }
    Some(features)
}

/// Bounds and content rules of the modern client feature selection.
fn valid_features(features: &[&str], feature_pattern: &Regex) -> bool {
    if features.len() < 2 || features.len() > 16 {
        return false;
    }
    if !features.iter().all(|item| feature_pattern.is_match(item)) {
        return false;
    }
    let unique: BTreeSet<&str> = features.iter().copied().collect();
    if unique.len() != features.len() {
        return false;
    }
    features.contains(&"default-https-client")
        && features.contains(&"rt-tokio")
        && !features.contains(&"rustls")
}
