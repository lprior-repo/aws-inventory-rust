//! Verification of fetched SDK `.crate` archives in the Cargo cache.
//!
//! Python's `tarfile` and `toml` modules have no workspace dependency here, so
//! the archive is read by GNU `tar`; every byte-level decision (bounds, digest,
//! feature equality) stays in Rust. Cargo archives are plain ustar+gzip.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::file_sha256;

use super::sdk_catalog::CacheReport;

/// Supported number of Cargo registry cache directories.
const MAX_REGISTRIES: usize = 16;
/// Archive members searched before giving up on the manifest.
const MAX_MEMBERS: usize = 128;
/// Largest accepted SDK package manifest.
const MAX_MANIFEST_BYTES: u64 = 1_048_576;
/// Read bound including the byte that proves the manifest is too large.
const MANIFEST_READ_BOUND: u64 = 1_048_577;

/// Verify every SDK archive and its declared client features.
pub fn verify_cache(document: &Value, cargo_home: &Path) -> Result<CacheReport, HarnessError> {
    let registries = registries(cargo_home)?;
    let services = document
        .get("services")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("invalid service list"))?;
    let mut total_bytes: u64 = 0;
    for service in services {
        let size = verify_service(service, &registries)?;
        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| HarnessError::invalid("SDK archive byte total overflow"))?;
    }

    let count = document
        .get("service_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| HarnessError::invalid("invalid service count"))?;
    Ok(CacheReport {
        verified_sdk_archives: count,
        verified_client_feature_sets: count,
        sdk_archive_bytes: total_bytes,
    })
}

/// The bounded set of Cargo registry cache directories.
fn registries(cargo_home: &Path) -> Result<Vec<PathBuf>, HarnessError> {
    let cache = cargo_home.join("registry").join("cache");
    let mut registries: Vec<PathBuf> = match fs::read_dir(&cache) {
        Ok(entries) => {
            let mut paths = Vec::new();
            for entry in entries {
                paths.push(entry?.path());
            }
            paths
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    registries.sort();
    if registries.is_empty() || registries.len() > MAX_REGISTRIES {
        return Err(HarnessError::invalid(
            "expected 1..=16 Cargo registry cache directories",
        ));
    }
    Ok(registries)
}

/// Verify one service archive and return its on-disk size.
fn verify_service(service: &Value, registries: &[PathBuf]) -> Result<u64, HarnessError> {
    let name = string_field(service, "name")?;
    let version = string_field(service, "version")?;
    let checksum = string_field(service, "checksum_sha256")?;
    let archive = format!("{name}-{version}.crate");
    for path in archive_candidates(registries, &archive)? {
        if file_sha256(&path)? != checksum {
            continue;
        }
        verify_client_features(&path, service)?;
        return Ok(fs::metadata(&path)?.len());
    }
    Err(HarnessError::invalid(format!(
        "SDK archive checksum mismatch: {archive}"
    )))
}

/// Every registry holding the named archive, failing when none does.
fn archive_candidates(registries: &[PathBuf], archive: &str) -> Result<Vec<PathBuf>, HarnessError> {
    let matches: Vec<PathBuf> = registries
        .iter()
        .map(|registry| registry.join(archive))
        .filter(|path| path.is_file())
        .collect();
    if matches.is_empty() {
        return Err(HarnessError::invalid(format!(
            "SDK archive missing from Cargo cache: {archive}; run cargo fetch --locked"
        )));
    }
    Ok(matches)
}

/// Read a required string field from a service.
fn string_field<'a>(service: &'a Value, name: &str) -> Result<&'a str, HarnessError> {
    service
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid(format!("service field is not a string: {name}")))
}

/// Locate the package manifest and compare its default features.
fn verify_client_features(archive: &Path, service: &Value) -> Result<(), HarnessError> {
    let name = string_field(service, "name")?;
    let version = string_field(service, "version")?;
    let manifest_name = format!("{name}-{version}/Cargo.toml");
    require_member(archive, &manifest_name)?;
    let defaults = default_features(archive, &manifest_name)?;
    let declared = declared_features(service);
    if !features_match(&defaults, &declared) {
        return Err(HarnessError::invalid(format!(
            "SDK client features differ from verified archive: {name}"
        )));
    }
    Ok(())
}

/// Fail unless the archive listing includes the package manifest.
fn require_member(archive: &Path, manifest_name: &str) -> Result<(), HarnessError> {
    let listing = run(Command::new("tar").arg("-tzf").arg(archive))?;
    let listing = String::from_utf8(listing)
        .map_err(|error| HarnessError::invalid(format!("tar listing is not UTF-8: {error}")))?;
    for (index, member) in listing.lines().enumerate() {
        if index >= MAX_MEMBERS {
            return Err(HarnessError::invalid(
                "SDK manifest exceeds archive search bound",
            ));
        }
        if member == manifest_name {
            return Ok(());
        }
    }
    Err(HarnessError::invalid(
        "SDK archive lacks its package manifest",
    ))
}

/// Default features declared by the archive, excluding the TLS choice.
fn default_features(
    archive: &Path,
    manifest_name: &str,
) -> Result<Vec<Option<String>>, HarnessError> {
    let bytes = extract(archive, manifest_name)?;
    let text = String::from_utf8(bytes)
        .map_err(|error| HarnessError::invalid(format!("SDK manifest is not UTF-8: {error}")))?;
    let manifest: toml::Value = toml::from_str(&text)
        .map_err(|error| HarnessError::invalid(format!("SDK manifest is not TOML: {error}")))?;
    let defaults = manifest
        .get("features")
        .and_then(|features| features.get("default"))
        .and_then(toml::Value::as_array);
    Ok(defaults
        .map(|items| {
            items
                .iter()
                .filter(|item| item.as_str() != Some("rustls"))
                .map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

/// Client features claimed by the snapshot service entry.
fn declared_features(service: &Value) -> Vec<String> {
    service
        .get("client_features")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Ordered feature equality, keeping non-string entries unequal.
fn features_match(defaults: &[Option<String>], declared: &[String]) -> bool {
    defaults.len() == declared.len()
        && defaults
            .iter()
            .zip(declared.iter())
            .all(|(left, right)| left.as_deref() == Some(right.as_str()))
}

/// Extract one bounded member of a `.crate` archive.
fn extract(archive: &Path, member: &str) -> Result<Vec<u8>, HarnessError> {
    let mut child = Command::new("tar")
        .arg("-xzOf")
        .arg(archive)
        .arg(member)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut bytes = Vec::new();
    {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HarnessError::invalid("failed to capture SDK archive member"))?;
        stdout.take(MANIFEST_READ_BOUND).read_to_end(&mut bytes)?;
    }
    let status = child.wait()?;
    check_manifest_size(bytes.len())?;
    if !status.success() {
        return Err(HarnessError::invalid(format!(
            "SDK archive member could not be read: {member}"
        )));
    }
    Ok(bytes)
}

/// Enforce the manifest size bounds on the extracted bytes.
fn check_manifest_size(len: usize) -> Result<(), HarnessError> {
    let size =
        u64::try_from(len).map_err(|_| HarnessError::invalid("SDK manifest size overflow"))?;
    if size == 0 || size > MAX_MANIFEST_BYTES {
        return Err(HarnessError::invalid(
            "SDK manifest is not a bounded regular file",
        ));
    }
    Ok(())
}

/// Run a command to completion and return stdout.
fn run(command: &mut Command) -> Result<Vec<u8>, HarnessError> {
    let output = command.stderr(Stdio::null()).output()?;
    if !output.status.success() {
        return Err(HarnessError::invalid(
            "SDK archive could not be listed; run cargo fetch --locked",
        ));
    }
    Ok(output.stdout)
}
