//! Deterministic ZIP writing and read-back verification.

use std::fs::File;
use std::io::{self, Read, Seek, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use super::ARCHIVE_PREFIX;
use super::Payload;
use super::selection::MANIFEST_NAME;
use super::sha256_hex;

/// Deflate level used by the original Python archive writer.
const COMPRESSION_LEVEL: i64 = 7;

/// Fixed ZIP timestamp (the earliest representable DOS time), so archives are
/// reproducible instead of inheriting file modification times.
pub(super) fn fixed_timestamp() -> Result<DateTime, Box<dyn std::error::Error>> {
    DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).map_err(zip_error)
}

/// Convert a ZIP failure into an operator-facing I/O error.
fn zip_error(error: impl std::fmt::Display) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("ZIP archive failure: {error}"),
    ))
}

/// Write the payload set and manifest, sorted by the caller.
pub(super) fn write(
    file: &mut File,
    payloads: &[Payload],
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(COMPRESSION_LEVEL))
        .last_modified_time(fixed_timestamp()?)
        .unix_permissions(0o644);
    payloads
        .iter()
        .try_for_each(|(name, data)| write_entry(&mut writer, name, data, options))?;
    write_entry(&mut writer, MANIFEST_NAME, manifest.as_bytes(), options)?;
    writer.finish().map_err(zip_error)?;
    Ok(())
}

/// Write one entry through the already exclusively opened archive handle.
fn write_entry(
    writer: &mut ZipWriter<&mut File>,
    name: &str,
    data: &[u8],
    options: SimpleFileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    writer
        .start_file(format!("{ARCHIVE_PREFIX}{name}"), options)
        .map_err(zip_error)?;
    writer.write_all(data)?;
    Ok(())
}

/// Re-read the archive, checking entry count, payload hashes and manifest bytes.
pub(super) fn verify(
    file: &mut File,
    payloads: &[Payload],
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    file.rewind()?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let expected = payloads
        .len()
        .checked_add(1)
        .ok_or_else(|| invalid("payload count exceeds supported bounds"))?;
    if archive.len() != expected {
        return Err(invalid("ZIP entry count mismatch"));
    }
    payloads
        .iter()
        .try_for_each(|(name, data)| verify_entry(&mut archive, name, data))?;
    let manifest_bytes = read_entry(&mut archive, &format!("{ARCHIVE_PREFIX}{MANIFEST_NAME}"))?;
    if manifest_bytes != manifest.as_bytes() {
        return Err(invalid("packaged manifest mismatch"));
    }
    Ok(())
}

/// Compare one archived payload with the input snapshot.
fn verify_entry(
    archive: &mut ZipArchive<&mut File>,
    name: &str,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = read_entry(archive, &format!("{ARCHIVE_PREFIX}{name}"))?;
    if sha256_hex(&bytes) != sha256_hex(data) {
        return Err(invalid(format!("packaged content hash mismatch: {name}")));
    }
    Ok(())
}

/// Read one named entry completely, validating its CRC as a side effect.
fn read_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut entry = archive.by_name(name).map_err(zip_error)?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Build an invalid-data error for consistency-check failures.
fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}
