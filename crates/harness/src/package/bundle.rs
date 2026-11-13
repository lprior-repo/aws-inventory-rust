//! Byte-parity port of the retired Python packager: source selection, ordering,
//! manifest bytes and ZIP entry payloads match; only container timestamps differ.

use std::fs::{self, File, Metadata};
use std::io::{Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde_json::{Map, Value};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::HarnessError;
use crate::provenance::{file_sha256, sha256_hex};

use super::selection::{MANIFEST_NAME, slash_path, walk};

/// Top-level directory inside the archive.
const ARCHIVE_PREFIX: &str = "aws-inventory-rs/";
/// Deflate level used by the Python archive writer.
const COMPRESSION_LEVEL: i64 = 7;

/// Receipt printed by the Python entry point after packaging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageReceipt {
    pub path: String,
    pub files: u64,
    pub bytes: u64,
    pub sha256: String,
}

impl PackageReceipt {
    /// JSON shape of `json.dumps(receipt, indent=2, sort_keys=True)`.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("path".to_string(), Value::String(self.path.clone()));
        map.insert("files".to_string(), Value::from(self.files));
        map.insert("bytes".to_string(), Value::from(self.bytes));
        map.insert("sha256".to_string(), Value::String(self.sha256.clone()));
        Value::Object(map)
    }
}

/// Package every distributable file under `root` into `destination`.
///
/// `MANIFEST.sha256` is written under `root` exactly as Python does. The
/// archive is built under a temporary sibling and atomically replaced.
pub fn package(root: &Path, destination: &Path) -> Result<PackageReceipt, HarnessError> {
    let paths = selected_paths(root)?;
    let payloads = read_payloads(root, &paths)?;
    let manifest = manifest_text(&payloads);
    fs::write(root.join(MANIFEST_NAME), manifest.as_bytes())?;
    write_verified(destination, &payloads, &manifest)?;
    receipt(destination, payloads.len())
}

/// Every distributable path, ordered by its slash-separated archive name.
fn selected_paths(root: &Path) -> Result<Vec<std::path::PathBuf>, HarnessError> {
    let mut paths = Vec::new();
    walk(root, root, &mut paths)?;
    paths.sort_by_cached_key(|path| slash_path(path));
    Ok(paths)
}

/// Read each selected file, failing if it changes mid-read.
fn read_payloads(
    root: &Path,
    paths: &[std::path::PathBuf],
) -> Result<Vec<(String, Vec<u8>)>, HarnessError> {
    let mut payloads: Vec<(String, Vec<u8>)> = Vec::with_capacity(paths.len());
    for relative in paths {
        let name = slash_path(relative);
        let absolute = root.join(relative);
        let before = fs::metadata(&absolute)?;
        let data = fs::read(&absolute)?;
        let after = fs::metadata(&absolute)?;
        if stamp(&before)? != stamp(&after)? {
            return Err(HarnessError::invalid(format!(
                "file changed during packaging: {name}"
            )));
        }
        payloads.push((name, data));
    }
    Ok(payloads)
}

/// `sha256  name` lines in archive order.
fn manifest_text(payloads: &[(String, Vec<u8>)]) -> String {
    let mut manifest = String::new();
    for (name, data) in payloads {
        manifest.push_str(&sha256_hex(data));
        manifest.push_str("  ");
        manifest.push_str(name);
        manifest.push('\n');
    }
    manifest
}

/// Build, re-verify and atomically replace the destination archive.
fn write_verified(
    destination: &Path,
    payloads: &[(String, Vec<u8>)],
    manifest: &str,
) -> Result<(), HarnessError> {
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = destination.with_extension("building.zip");
    write_archive(&temporary, payloads, manifest)?;
    verify_archive(&temporary, payloads, manifest)?;
    fs::rename(&temporary, destination)?;
    Ok(())
}

fn receipt(destination: &Path, payload_count: usize) -> Result<PackageReceipt, HarnessError> {
    let resolved = fs::canonicalize(destination)?;
    let path = resolved
        .to_str()
        .ok_or_else(|| HarnessError::invalid("packaged path is not valid UTF-8"))?
        .to_string();
    let files = u64::try_from(payload_count)
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| HarnessError::invalid("payload count exceeds supported bounds"))?;
    Ok(PackageReceipt {
        path,
        files,
        bytes: fs::metadata(destination)?.len(),
        sha256: file_sha256(destination)?,
    })
}

/// Fields Python compares before and after reading a file.
fn stamp(metadata: &Metadata) -> Result<(u64, Option<u128>), HarnessError> {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_nanos());
    Ok((metadata.len(), modified))
}

/// Convert a ZIP failure into the harness's operator-facing failure.
fn zip_error(error: zip::result::ZipError) -> HarnessError {
    HarnessError::invalid(format!("ZIP archive failure: {error}"))
}

/// Write the payload set and manifest with the Python compression settings.
fn write_archive(
    path: &Path,
    payloads: &[(String, Vec<u8>)],
    manifest: &str,
) -> Result<(), HarnessError> {
    let file = File::create(path)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(COMPRESSION_LEVEL))
        .unix_permissions(0o644);
    for (name, data) in payloads {
        writer
            .start_file(format!("{ARCHIVE_PREFIX}{name}"), options)
            .map_err(zip_error)?;
        writer.write_all(data)?;
    }
    writer
        .start_file(format!("{ARCHIVE_PREFIX}{MANIFEST_NAME}"), options)
        .map_err(zip_error)?;
    writer.write_all(manifest.as_bytes())?;
    writer.finish().map_err(zip_error)?;
    Ok(())
}

/// Re-read the archive, checking CRC and payload bytes like the Python check.
fn verify_archive(
    path: &Path,
    payloads: &[(String, Vec<u8>)],
    manifest: &str,
) -> Result<(), HarnessError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let expected = payloads
        .len()
        .checked_add(1)
        .ok_or_else(|| HarnessError::invalid("payload count exceeds supported bounds"))?;
    if archive.len() != expected {
        return Err(HarnessError::invalid("ZIP entry count mismatch"));
    }
    for (name, data) in payloads {
        verify_entry(&mut archive, name, data)?;
    }
    verify_manifest(&mut archive, manifest)
}

/// Compare one re-read payload entry by digest.
fn verify_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
    data: &[u8],
) -> Result<(), HarnessError> {
    let entry_name = format!("{ARCHIVE_PREFIX}{name}");
    let mut entry = archive.by_name(&entry_name).map_err(zip_error)?;
    let mut buffer = Vec::new();
    entry.read_to_end(&mut buffer)?;
    if sha256_hex(&buffer) != sha256_hex(data) {
        return Err(HarnessError::invalid(format!(
            "packaged content hash mismatch: {name}"
        )));
    }
    Ok(())
}

/// Compare the re-read manifest entry against the written bytes.
fn verify_manifest(archive: &mut ZipArchive<File>, manifest: &str) -> Result<(), HarnessError> {
    let mut entry = archive
        .by_name(&format!("{ARCHIVE_PREFIX}{MANIFEST_NAME}"))
        .map_err(zip_error)?;
    let mut buffer = Vec::new();
    entry.read_to_end(&mut buffer)?;
    if buffer != manifest.as_bytes() {
        return Err(HarnessError::invalid("packaged manifest mismatch"));
    }
    Ok(())
}
