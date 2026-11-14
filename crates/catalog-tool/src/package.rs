//! Reproducible release packaging without Python.
//!
//! Selection, ordering, manifest bytes and ZIP payloads match the retired
//! the retired Python packager. The deliberate difference is determinism: every ZIP
//! entry carries the fixed 1980-01-01 timestamp instead of the packaging time.

mod archive;
mod destination;
mod limits;
mod selection;

use std::fs::{File, Metadata};
use std::io::{self, Read, Seek};
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use limits::Limits;
use selection::{MANIFEST_NAME, slash_path, walk};

pub use destination::atomic_write;

/// Top-level directory inside the archive.
const ARCHIVE_PREFIX: &str = "aws-inventory-rs/";

/// Fixed working storage for streaming the completed archive digest.
const HASH_BUFFER_BYTES: usize = 8192;

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
/// archive is built in an exclusive temporary sibling, verified, then renamed.
/// Destinations inside the canonical root are rejected before any writes with
/// an `io::ErrorKind::InvalidInput` error naming both paths. Existing external
/// files are replaced; destination symlinks themselves are replaced, not followed.
/// The destination directory must not be writable by untrusted users; validation is not atomic with publication.
pub fn package(
    root: &Path,
    destination: &Path,
) -> Result<PackageReceipt, Box<dyn std::error::Error>> {
    let (root, destination) = destination::validate(root, destination)?;
    let payloads = collect_payloads(&root, limits::DEFAULT)?;
    let manifest = build_manifest(&payloads);
    atomic_write(&root.join(MANIFEST_NAME), |file| {
        use std::io::Write;
        file.write_all(manifest.as_bytes())?;
        Ok(())
    })?;
    atomic_write(&destination, |file| {
        archive::write(file, &payloads, &manifest)?;
        archive::verify(file, &payloads, &manifest)?;
        receipt(&destination, file, payloads.len())
    })
}

/// Read included files within named traversal and metadata-checked byte limits.
fn collect_payloads(
    root: &Path,
    limits: Limits,
) -> Result<Vec<Payload>, Box<dyn std::error::Error>> {
    let mut paths = walk(root, limits)?;
    paths.sort_by_cached_key(|path| slash_path(path));
    let mut payloads = Vec::new();
    payloads.try_reserve_exact(paths.len())?;
    paths.iter().try_fold(0, |total, relative| {
        let (payload, next) = read_payload(root, relative, total, limits)?;
        payloads.push(payload);
        Ok::<_, Box<dyn std::error::Error>>(next)
    })?;
    Ok(payloads)
}

/// Check metadata byte budgets before allocating or reading a payload.
fn read_payload(
    root: &Path,
    relative: &Path,
    total: u64,
    limits: Limits,
) -> Result<(Payload, u64), Box<dyn std::error::Error>> {
    let absolute = root.join(relative);
    let mut file = File::open(&absolute)?;
    let before = file.metadata()?;
    let next = limits.bytes(&absolute, before.len(), total)?;
    let data = read_snapshot(&mut file, before.len())?;
    if stamp(&before)? != stamp(&file.metadata()?)? {
        return Err(invalid(format!(
            "file changed during packaging: {}",
            absolute.display()
        )));
    }
    Ok(((slash_path(relative), data), next))
}

/// A growing file cannot grow this allocation beyond its checked metadata size.
fn read_snapshot(file: &mut File, length: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let length = usize::try_from(length)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length)?;
    bytes.resize(length, 0);
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Build the manifest text: `<sha256>  <slash path>\n` per payload, in order.
fn build_manifest(payloads: &[Payload]) -> String {
    payloads
        .iter()
        .fold(String::new(), |mut manifest, (name, data)| {
            manifest.push_str(&sha256_hex(data));
            manifest.push_str("  ");
            manifest.push_str(name);
            manifest.push('\n');
            manifest
        })
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
    file: &mut File,
    payload_count: usize,
) -> Result<PackageReceipt, Box<dyn std::error::Error>> {
    let path = destination
        .to_str()
        .ok_or_else(|| invalid("packaged path is not valid UTF-8"))?
        .to_string();
    let files = u64::try_from(payload_count)
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| invalid("payload count exceeds supported bounds"))?;
    let sha256 = hash_archive(file)?;
    Ok(PackageReceipt {
        path,
        files,
        bytes: file.metadata()?.len(),
        sha256,
    })
}

/// Lowercase hexadecimal SHA-256 of a byte slice.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Hash the completed, private archive with a fixed 8 KiB working buffer.
fn hash_archive(file: &mut File) -> io::Result<String> {
    file.rewind()?;
    let mut digest = Sha256::new();
    let mut buffer = [0; HASH_BUFFER_BYTES];
    // Each successful read advances through a finite, no-longer-written file.
    std::iter::from_fn(|| hash_next(file, &mut buffer, &mut digest))
        .try_for_each(std::convert::identity)?;
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Hash one chunk; EOF terminates iteration and read errors propagate.
fn hash_next(file: &mut File, buffer: &mut [u8], digest: &mut Sha256) -> Option<io::Result<()>> {
    match file.read(buffer) {
        Ok(0) => None,
        Ok(count) => Some(
            buffer
                .get(..count)
                .ok_or_else(|| io::Error::other("archive read exceeded hash buffer"))
                .map(|bytes| digest.update(bytes)),
        ),
        Err(error) => Some(Err(error)),
    }
}

/// Build an invalid-data error for consistency-check failures.
fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod safety_tests;

#[cfg(test)]
mod limits_tests;

#[cfg(all(test, unix))]
mod publication_tests;
