//! Offline SDK catalog reproduction: check, regenerate and cache verification.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::HarnessError;

use super::{sdk_cache, sdk_generate, sdk_validate};

/// Relative location of the generated catalog crate.
const DESTINATION: &str = "crates/sdk-catalog";
/// Snapshot file name inside the catalog crate.
const SNAPSHOT: &str = "sdk-services.json";
/// Original upstream manifest file name inside the catalog crate.
const UPSTREAM: &str = "upstream-versions.toml";

/// Whether generated files are compared or written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdkMode {
    /// Verify committed files without changing them.
    Check,
    /// Rewrite generated files from the snapshot.
    Generate,
}

impl SdkMode {
    /// Mode label printed by the Python entry point.
    pub fn label(self) -> &'static str {
        match self {
            Self::Check => "checked",
            Self::Generate => "generated",
        }
    }
}

/// Totals reported by the optional Cargo cache verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheReport {
    /// Number of verified SDK archives.
    pub verified_sdk_archives: u64,
    /// Number of verified client feature sets.
    pub verified_client_feature_sets: u64,
    /// Total bytes of verified archives.
    pub sdk_archive_bytes: u64,
}

/// Result of one offline catalog operation.
#[derive(Debug, Clone)]
pub struct SdkReport {
    /// Number of services in the validated snapshot.
    pub service_count: u64,
    /// Official release tag, or JSON null when absent.
    pub release: Value,
    /// Services whose checked pin is not the latest published version.
    pub newer_registry_versions: Vec<String>,
    /// Whether files were checked or generated.
    pub mode: SdkMode,
    /// Cache verification totals when requested.
    pub cache: Option<CacheReport>,
}

impl SdkReport {
    /// JSON shape of the Python entry point's success line.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("service_count".to_string(), Value::from(self.service_count));
        map.insert("release".to_string(), self.release.clone());
        let versions = self
            .newer_registry_versions
            .iter()
            .map(|name| Value::String(name.clone()))
            .collect();
        map.insert(
            "newer_registry_versions".to_string(),
            Value::Array(versions),
        );
        map.insert(
            "mode".to_string(),
            Value::String(self.mode.label().to_string()),
        );
        if let Some(cache) = &self.cache {
            insert_cache(&mut map, cache);
        }
        Value::Object(map)
    }
}

/// Append the optional Cargo cache totals in the Python key order.
fn insert_cache(map: &mut Map<String, Value>, cache: &CacheReport) {
    map.insert(
        "verified_sdk_archives".to_string(),
        Value::from(cache.verified_sdk_archives),
    );
    map.insert(
        "verified_client_feature_sets".to_string(),
        Value::from(cache.verified_client_feature_sets),
    );
    map.insert(
        "sdk_archive_bytes".to_string(),
        Value::from(cache.sdk_archive_bytes),
    );
}

/// Run the offline catalog operation, using the ambient Cargo cache.
pub fn sdk_run(root: &Path, mode: SdkMode, verify_cache: bool) -> Result<SdkReport, HarnessError> {
    sdk_run_in(root, mode, verify_cache, None)
}

/// Run the offline catalog operation with an explicit Cargo home for testing.
pub fn sdk_run_in(
    root: &Path,
    mode: SdkMode,
    verify_cache: bool,
    cargo_home: Option<&Path>,
) -> Result<SdkReport, HarnessError> {
    let destination = root.join(DESTINATION);
    let document = load_document(&destination)?;
    let outputs = sdk_generate::outputs(&destination, &document)?;
    for (path, text) in &outputs {
        match mode {
            SdkMode::Generate => sdk_generate::write(path, text)?,
            SdkMode::Check => check_file(root, path, text)?,
        }
    }
    let services = services(&document)?;
    let cache = cache_report(verify_cache, cargo_home, &document)?;
    report(&document, mode, newer_versions(services), cache)
}

/// Read and validate the committed snapshot and upstream manifest.
fn load_document(destination: &Path) -> Result<Value, HarnessError> {
    let snapshot = fs::read(destination.join(SNAPSHOT))?;
    let original = fs::read(destination.join(UPSTREAM))?;
    let document: Value = serde_json::from_slice(&snapshot)?;
    sdk_validate::validate(&document, &original)?;
    Ok(document)
}

/// Snapshot services array, already validated by the caller.
fn services(document: &Value) -> Result<&[Value], HarnessError> {
    document
        .get("services")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| HarnessError::invalid("invalid service list"))
}

/// Services whose checked pin is older than the latest published version.
fn newer_versions(services: &[Value]) -> Vec<String> {
    let mut newer = Vec::new();
    for service in services {
        let version = service.get("version").and_then(Value::as_str);
        let latest = service
            .get("latest_published_version_at_check")
            .and_then(Value::as_str);
        if version != latest
            && let Some(name) = service.get("name").and_then(Value::as_str)
        {
            newer.push(name.to_string());
        }
    }
    newer
}

/// Verify the Cargo cache when requested, resolving the ambient Cargo home.
fn cache_report(
    verify_cache: bool,
    cargo_home: Option<&Path>,
    document: &Value,
) -> Result<Option<CacheReport>, HarnessError> {
    if !verify_cache {
        return Ok(None);
    }
    let home = match cargo_home {
        Some(home) => home.to_path_buf(),
        None => ambient_cargo_home()?,
    };
    Ok(Some(sdk_cache::verify_cache(document, &home)?))
}

/// Assemble the report from the validated snapshot.
fn report(
    document: &Value,
    mode: SdkMode,
    newer: Vec<String>,
    cache: Option<CacheReport>,
) -> Result<SdkReport, HarnessError> {
    let service_count = document
        .get("service_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| HarnessError::invalid("invalid service count"))?;
    Ok(SdkReport {
        service_count,
        release: document
            .get("official_release_tag")
            .cloned()
            .unwrap_or(Value::Null),
        newer_registry_versions: newer,
        mode,
        cache,
    })
}

/// Verify the committed catalog without changing it.
pub fn sdk_check(root: &Path) -> Result<SdkReport, HarnessError> {
    sdk_run(root, SdkMode::Check, false)
}

/// Regenerate the committed catalog from the snapshot.
pub fn sdk_reproduce(root: &Path) -> Result<SdkReport, HarnessError> {
    sdk_run(root, SdkMode::Generate, false)
}

/// Regenerate and additionally verify the Cargo cache archives.
pub fn sdk_verify_cache(root: &Path) -> Result<SdkReport, HarnessError> {
    sdk_run(root, SdkMode::Generate, true)
}

/// Network refresh is intentionally absent from the Rust harness.
pub fn sdk_refresh() -> Result<SdkReport, HarnessError> {
    Err(HarnessError::invalid(
        "network refresh is not implemented in the Rust harness",
    ))
}

/// Compare one generated file, mirroring Python's `read_text` check.
fn check_file(root: &Path, path: &Path, text: &str) -> Result<(), HarnessError> {
    let current = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if current.as_deref() != Some(text.as_bytes()) {
        return Err(HarnessError::invalid(format!(
            "generated file differs: {}",
            relative_name(root, path)?
        )));
    }
    Ok(())
}

/// Root-relative slash path used in parity error messages.
fn relative_name(root: &Path, path: &Path) -> Result<String, HarnessError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| HarnessError::invalid("generated path is outside the workspace"))?;
    let mut name = String::new();
    for component in relative.components() {
        if let std::path::Component::Normal(part) = component {
            if !name.is_empty() {
                name.push('/');
            }
            name.push_str(&part.to_string_lossy());
        }
    }
    Ok(name)
}

/// Resolve Cargo's home directory exactly as Python's `Path.home()` fallback.
fn ambient_cargo_home() -> Result<PathBuf, HarnessError> {
    if let Ok(value) = std::env::var("CARGO_HOME")
        && !value.is_empty()
    {
        return Ok(PathBuf::from(value));
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return Ok(PathBuf::from(home).join(".cargo"));
    }
    Err(HarnessError::invalid(
        "neither CARGO_HOME nor HOME identifies a Cargo home",
    ))
}
