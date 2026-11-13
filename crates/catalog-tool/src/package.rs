//! Reproducible release packaging without Python.
//!
//! Selection, ordering, manifest bytes and ZIP payloads match the retired
//! the retired Python packager. The deliberate difference is determinism: every ZIP
//! entry carries the fixed 1980-01-01 timestamp instead of the packaging time.

mod archive;
mod selection;

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use selection::{MANIFEST_NAME, slash_path, walk};

/// Top-level directory inside the archive.
const ARCHIVE_PREFIX: &str = "aws-inventory-rs/";

/// One included file: slash-separated relative name and exact bytes.
pub(super) type Payload = (String, Vec<u8>);

/// Receipt printed after a successful package, matching the Python JSON shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageReceipt {
    pub path: String,
    pub files: u64,
    pub bytes: u64,
    pub sha256: String,
}

impl PackageReceipt {
    /// Sorted-key JSON object, as `json.dumps(..., sort_keys=True)` produced.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("path".to_string(), Value::String(self.path.clone()));
        map.insert("files".to_string(), Value::from(self.files));
        map.insert("bytes".to_string(), Value::from(self.bytes));
        map.insert("sha256".to_string(), Value::String(self.sha256.clone()));
        Value::Object(map)
    }
}

/// Package `root` into `destination` and print the receipt summary.
pub fn run(root: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let receipt = package(root, destination)?;
    let summary = serde_json::to_string_pretty(&receipt.to_json())?;
    println!("{summary}");
    Ok(())
}

/// Package every distributable file under `root` into `destination`.
///
/// `MANIFEST.sha256` is written under `root`, matching the Python tool. The
/// archive is built under a temporary sibling, verified, then renamed.
pub fn package(
    root: &Path,
    destination: &Path,
) -> Result<PackageReceipt, Box<dyn std::error::Error>> {
    let payloads = collect_payloads(root)?;
    let manifest = build_manifest(&payloads);
    fs::write(root.join(MANIFEST_NAME), manifest.as_bytes())?;
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = destination.with_extension("building.zip");
    archive::write(&temporary, &payloads, &manifest)?;
    archive::verify(&temporary, &payloads, &manifest)?;
    fs::rename(&temporary, destination)?;
    receipt(destination, payloads.len())
}

/// Read every included file, rejecting files that change mid-read.
fn collect_payloads(root: &Path) -> Result<Vec<Payload>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    walk(root, root, &mut paths)?;
    paths.sort_by_cached_key(|path| slash_path(path));
    let mut payloads = Vec::with_capacity(paths.len());
    for relative in &paths {
        let name = slash_path(relative);
        let absolute = root.join(relative);
        let before = fs::metadata(&absolute)?;
        let data = fs::read(&absolute)?;
        let after = fs::metadata(&absolute)?;
        if stamp(&before)? != stamp(&after)? {
            return Err(invalid(format!("file changed during packaging: {name}")));
        }
        payloads.push((name, data));
    }
    Ok(payloads)
}

/// Build the manifest text: `<sha256>  <slash path>\n` per payload, in order.
fn build_manifest(payloads: &[Payload]) -> String {
    let mut manifest = String::new();
    for (name, data) in payloads {
        manifest.push_str(&sha256_hex(data));
        manifest.push_str("  ");
        manifest.push_str(name);
        manifest.push('\n');
    }
    manifest
}

/// Fields the Python tool compares before and after reading a file.
fn stamp(metadata: &Metadata) -> io::Result<(u64, Option<u128>)> {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_nanos());
    Ok((metadata.len(), modified))
}

/// Canonical destination plus payload count and archive digest.
fn receipt(
    destination: &Path,
    payload_count: usize,
) -> Result<PackageReceipt, Box<dyn std::error::Error>> {
    let resolved: PathBuf = fs::canonicalize(destination)?;
    let path = resolved
        .to_str()
        .ok_or_else(|| invalid("packaged path is not valid UTF-8"))?
        .to_string();
    let files = u64::try_from(payload_count)
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| invalid("payload count exceeds supported bounds"))?;
    Ok(PackageReceipt {
        path,
        files,
        bytes: fs::metadata(destination)?.len(),
        sha256: file_sha256(destination)?,
    })
}

/// Lowercase hexadecimal SHA-256 of a byte slice.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Lowercase hexadecimal SHA-256 of a file's bytes.
pub(crate) fn file_sha256(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(sha256_hex(&fs::read(path)?))
}

/// Build an invalid-data error for consistency-check failures.
fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}

#[cfg(test)]
mod tests;
