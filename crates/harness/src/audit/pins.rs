//! Pinned SDK operation modules and credential-provider source evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::HarnessError;
use crate::files::read_bytes;
use crate::provenance::file_sha256;

use super::files::{MAX_INPUT_BYTES, array_field, text_field};
use super::operations::additions::{AdditionRow, Additions};

/// Slack allowed on Cargo registry source roots.
const MAX_REGISTRY_ROOTS: usize = 32;

fn registry_home() -> Result<PathBuf, HarnessError> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or_else(|| HarnessError::invalid("Cargo registry home is unavailable"))
}

fn registry_roots(home: &Path) -> Result<Vec<PathBuf>, HarnessError> {
    let mut roots = Vec::new();
    for entry in fs::read_dir(home.join("registry").join("src"))? {
        roots.push(entry?.path());
    }
    roots.sort();
    if roots.len() > MAX_REGISTRY_ROOTS {
        return Err(HarnessError::invalid(
            "Cargo registry source root budget exceeded",
        ));
    }
    Ok(roots)
}

/// Locate the single cached registry copy of a pinned source file.
pub fn registry_file(
    crate_name: &str,
    version: &str,
    relative: &str,
) -> Result<PathBuf, HarnessError> {
    let roots = registry_roots(&registry_home()?)?;
    let directory = format!("{crate_name}-{version}");
    let matches: Vec<PathBuf> = roots
        .iter()
        .map(|root| root.join(&directory).join(relative))
        .filter(|path| path.is_file())
        .collect();
    match matches.as_slice() {
        [path] => Ok(path.clone()),
        _ => Err(HarnessError::invalid(format!(
            "expected one cached pinned SDK source: {crate_name} {version} {relative}"
        ))),
    }
}

fn package_identity(package: &toml::Value) -> Result<(&str, &str), HarnessError> {
    let name = package
        .get("name")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| HarnessError::invalid("Cargo.lock package identity"))?;
    let version = package
        .get("version")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| HarnessError::invalid("Cargo.lock package version"))?;
    Ok((name, version))
}

fn lock_versions(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, HarnessError> {
    let raw = read_bytes(&root.join("Cargo.lock"), MAX_INPUT_BYTES)?;
    let text =
        String::from_utf8(raw).map_err(|_| HarnessError::invalid("Cargo.lock is not UTF-8"))?;
    let lock: toml::Value = toml::from_str(&text)
        .map_err(|error| HarnessError::invalid(format!("Cargo.lock: {error}")))?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| HarnessError::invalid("Cargo.lock has no package list"))?;
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in packages {
        let (name, version) = package_identity(package)?;
        versions
            .entry(name.to_owned())
            .or_default()
            .insert(version.to_owned());
    }
    Ok(versions)
}

fn require_pinned(
    versions: &BTreeMap<String, BTreeSet<String>>,
    crate_name: &str,
    version: &str,
) -> Result<(), HarnessError> {
    if versions.get(crate_name) != Some(&BTreeSet::from([version.to_owned()])) {
        return Err(HarnessError::invalid(format!(
            "pinned SDK version changed: {crate_name}"
        )));
    }
    Ok(())
}

fn module_digest(
    operation: &Value,
    crate_name: &str,
    version: &str,
) -> Result<String, HarnessError> {
    let expected = operation
        .get("wire_source")
        .ok_or_else(|| HarnessError::invalid("missing wire_source"))?;
    let module = text_field(expected, "rust_operation_module")?;
    let digest = file_sha256(&registry_file(crate_name, version, module)?)?;
    if Some(digest.as_str())
        != expected
            .get("rust_operation_module_sha256")
            .and_then(Value::as_str)
    {
        return Err(HarnessError::invalid(format!(
            "pinned SDK operation module changed: {}",
            text_field(operation, "id")?
        )));
    }
    Ok(digest)
}

fn operation_row(
    operation: &Value,
    crate_name: &str,
    version: &str,
) -> Result<Value, HarnessError> {
    let digest = module_digest(operation, crate_name, version)?;
    Ok(json!({
        "id": text_field(operation, "id")?,
        "sdk_crate": crate_name,
        "sdk_version": version,
        "api_operation": text_field(operation, "api_operation")?,
        "module_sha256": digest,
    }))
}

fn operation_rows(
    values: &[Value],
    versions: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<Value>, HarnessError> {
    let mut rows = Vec::new();
    for operation in values {
        let crate_name = text_field(operation, "sdk_crate")?;
        let version = text_field(operation, "sdk_version")?;
        require_pinned(versions, crate_name, version)?;
        rows.push(operation_row(operation, crate_name, version)?);
    }
    Ok(rows)
}

fn provider_source(matrix: &Value) -> Result<&Map<String, Value>, HarnessError> {
    matrix
        .get("credential_provider_boundaries")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("provider_source"))
        .and_then(Value::as_object)
        .ok_or_else(|| HarnessError::invalid("credential-provider boundaries are missing"))
}

fn provider_fields(provider: &Map<String, Value>) -> Result<(&str, &str, &str), HarnessError> {
    let provider_crate = provider
        .get("crate")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("credential-provider crate"))?;
    let provider_version = provider
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("credential-provider version"))?;
    let provider_path = provider
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("credential-provider path"))?;
    Ok((provider_crate, provider_version, provider_path))
}

fn provider_evidence(
    provider: &Map<String, Value>,
    versions: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Value, HarnessError> {
    let (provider_crate, provider_version, provider_path) = provider_fields(provider)?;
    if versions.get(provider_crate) != Some(&BTreeSet::from([provider_version.to_owned()])) {
        return Err(HarnessError::invalid(
            "pinned credential provider version changed",
        ));
    }
    let digest = file_sha256(&registry_file(
        provider_crate,
        provider_version,
        provider_path,
    )?)?;
    if Some(digest.as_str()) != provider.get("sha256").and_then(Value::as_str) {
        return Err(HarnessError::invalid(
            "pinned credential provider source changed",
        ));
    }
    let mut evidence = provider.clone();
    evidence.insert("current_sha256".to_owned(), Value::String(digest));
    Ok(Value::Object(evidence))
}

/// Verify pinned SDK operation modules and the credential-provider source.
pub fn dependency_evidence(
    root: &Path,
    matrix: &Value,
) -> Result<(Vec<Value>, Value), HarnessError> {
    let versions = lock_versions(root)?;
    let rows = operation_rows(array_field(matrix, "operations")?, &versions)?;
    let evidence = provider_evidence(provider_source(matrix)?, &versions)?;
    Ok((rows, evidence))
}

/// Verify pinned SDK operation modules for reviewed additive operations.
pub fn addition_evidence(root: &Path, additions: &Additions) -> Result<Vec<Value>, HarnessError> {
    if !additions.is_present() {
        return Ok(Vec::new());
    }
    let versions = lock_versions(root)?;
    additions
        .rows
        .iter()
        .map(|row| addition_row(row, &versions))
        .collect()
}

fn addition_row(
    addition: &AdditionRow,
    versions: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Value, HarnessError> {
    let version = &addition.key.1;
    require_pinned(versions, &addition.crate_name, version)?;
    let module = format!("src/operation/{}.rs", addition.module);
    let digest = file_sha256(&registry_file(&addition.crate_name, version, &module)?)?;
    if digest != addition.module_sha256 {
        return Err(HarnessError::invalid(format!(
            "pinned SDK operation module changed: {}",
            addition.id
        )));
    }
    Ok(json!({
        "id": addition.id,
        "sdk_crate": addition.crate_name,
        "sdk_version": version,
        "api_operation": addition.api_operation,
        "module_sha256": digest,
    }))
}
