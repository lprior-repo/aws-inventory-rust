//! Exact snapshot validation from `generate_sdk_catalog.validate()`.

use std::collections::BTreeSet;

use regex::Regex;
use serde_json::Value;

use crate::HarnessError;
use crate::provenance::sha256_hex;

mod service;

use service::validate_service;

/// Hard bound on the official service count.
pub const MAX_SERVICES: usize = 1_024;
/// Official repository used to build origins.
pub const REPOSITORY: &str = "https://github.com/awslabs/aws-sdk-rust";
/// Prefix every registry crate name carries.
pub(super) const CRATE_PREFIX: &str = "aws-sdk-";

pub(super) struct Patterns {
    pub(super) name: Regex,
    pub(super) version: Regex,
    pub(super) hex: Regex,
    pub(super) commit: Regex,
    pub(super) feature: Regex,
}

impl Patterns {
    fn new() -> Result<Self, HarnessError> {
        Ok(Self {
            name: pattern(r"aws-sdk-[a-z][a-z0-9]*\z")?,
            version: pattern(r"[0-9]+\.[0-9]+\.[0-9]+\z")?,
            hex: pattern(r"[0-9a-f]{64}\z")?,
            commit: pattern(r"[0-9a-f]{40}\z")?,
            feature: pattern(r"[a-zA-Z0-9_-]+\z")?,
        })
    }
}

/// Compile a regular expression, surfacing the static pattern as invalid input.
fn pattern(expression: &str) -> Result<Regex, HarnessError> {
    Regex::new(expression).map_err(|error| HarnessError::invalid(error.to_string()))
}

/// Validate the snapshot against its original `upstream-versions.toml` bytes.
pub fn validate(document: &Value, original: &[u8]) -> Result<(), HarnessError> {
    let patterns = Patterns::new()?;
    let services = service_rows(document)?;
    check_schema(document, services)?;
    check_digest(document, original)?;
    let revision = check_revision(document, &patterns.commit)?;
    let (upstream, official_names) = upstream_names(original, &patterns.name)?;
    check_names(services, &official_names)?;
    for service in services {
        validate_service(service, document, &upstream, revision, &patterns)?;
    }
    Ok(())
}

fn service_rows(document: &Value) -> Result<&[Value], HarnessError> {
    let services = document
        .get("services")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("invalid service list"))?;
    if services.is_empty() || services.len() > MAX_SERVICES {
        return Err(HarnessError::invalid("invalid service list"));
    }
    Ok(services)
}

fn check_schema(document: &Value, services: &[Value]) -> Result<(), HarnessError> {
    let count =
        u64::try_from(services.len()).map_err(|_| HarnessError::invalid("invalid service list"))?;
    let schema_matches = document.get("schema_version").and_then(Value::as_u64) == Some(1)
        && document.get("service_count").and_then(Value::as_u64) == Some(count);
    if !schema_matches {
        return Err(HarnessError::invalid(
            "catalog schema or count does not match",
        ));
    }
    Ok(())
}

fn check_digest(document: &Value, original: &[u8]) -> Result<(), HarnessError> {
    let expected_digest = sha256_hex(original);
    if document
        .get("official_versions_sha256")
        .and_then(Value::as_str)
        != Some(expected_digest.as_str())
    {
        return Err(HarnessError::invalid(
            "upstream versions.toml checksum does not match the snapshot",
        ));
    }
    Ok(())
}

fn check_revision<'a>(
    document: &'a Value,
    commit_pattern: &Regex,
) -> Result<&'a str, HarnessError> {
    let revision = document
        .get("official_revision")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            HarnessError::invalid("snapshot must identify the immutable official Git commit")
        })?;
    if !commit_pattern.is_match(revision) {
        return Err(HarnessError::invalid(
            "snapshot must identify the immutable official Git commit",
        ));
    }
    check_manifest_url(document, revision)?;
    Ok(revision)
}

fn check_manifest_url(document: &Value, revision: &str) -> Result<(), HarnessError> {
    let expected_url =
        format!("https://raw.githubusercontent.com/awslabs/aws-sdk-rust/{revision}/versions.toml");
    if document
        .get("official_versions_url")
        .and_then(Value::as_str)
        != Some(expected_url.as_str())
    {
        return Err(HarnessError::invalid(
            "unexpected official version manifest origin",
        ));
    }
    Ok(())
}

fn upstream_names(
    original: &[u8],
    name_pattern: &Regex,
) -> Result<(toml::Table, BTreeSet<String>), HarnessError> {
    let text = std::str::from_utf8(original)
        .map_err(|error| HarnessError::invalid(format!("upstream versions.toml: {error}")))?;
    let mut upstream_document: toml::Value = toml::from_str(text)
        .map_err(|error| HarnessError::invalid(format!("upstream versions.toml: {error}")))?;
    let upstream = upstream_document
        .get_mut("crates")
        .and_then(toml::Value::as_table_mut)
        .map(std::mem::take)
        .ok_or_else(|| HarnessError::invalid("upstream versions.toml lacks crates"))?;
    let names = upstream
        .keys()
        .filter(|name| name_pattern.is_match(name))
        .cloned()
        .collect();
    Ok((upstream, names))
}

fn check_names(services: &[Value], official_names: &BTreeSet<String>) -> Result<(), HarnessError> {
    let names = service_names(services)?;
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    if names != sorted {
        return Err(HarnessError::invalid(
            "service names must be unique, sorted, and complete for the official source",
        ));
    }
    let observed: BTreeSet<String> = names.iter().map(|name| (*name).to_owned()).collect();
    if &observed != official_names {
        return Err(HarnessError::invalid(
            "service names must be unique, sorted, and complete for the official source",
        ));
    }
    Ok(())
}

fn service_names(services: &[Value]) -> Result<Vec<&str>, HarnessError> {
    let mut names: Vec<&str> = Vec::with_capacity(services.len());
    for service in services {
        let name = service.get("name").and_then(Value::as_str).ok_or_else(|| {
            HarnessError::invalid(
                "service names must be unique, sorted, and complete for the official source",
            )
        })?;
        names.push(name);
    }
    Ok(names)
}
