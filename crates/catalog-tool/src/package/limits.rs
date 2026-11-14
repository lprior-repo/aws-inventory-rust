//! Provisional resource ceilings for developer release packaging.

use std::io;
use std::path::Path;

const MAX_DIRECTORY_DEPTH: usize = 64;
const MAX_VISITED_ENTRIES: usize = 100_000;
const MAX_FILE_BYTES: u64 = 67_108_864;
const MAX_PAYLOAD_BYTES: u64 = 1_073_741_824;

/// Root-relative directory depth, visited entries and retained payload bytes.
#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub depth: usize,
    pub entries: usize,
    pub file_bytes: u64,
    pub total_bytes: u64,
}

/// Provisional ceilings: 64 levels, 100,000 entries, 64 MiB/file and 1 GiB total.
pub(super) const DEFAULT: Limits = Limits {
    depth: MAX_DIRECTORY_DEPTH,
    entries: MAX_VISITED_ENTRIES,
    file_bytes: MAX_FILE_BYTES,
    total_bytes: MAX_PAYLOAD_BYTES,
};

impl Limits {
    /// Check metadata before reserving or reading the next payload's bytes.
    pub(super) fn bytes(self, path: &Path, size: u64, total: u64) -> io::Result<u64> {
        if size > self.file_bytes {
            return Err(exceeded(path, "per-file bytes", self.file_bytes));
        }
        let next = total
            .checked_add(size)
            .ok_or_else(|| exceeded(path, "total payload bytes", self.total_bytes))?;
        if next > self.total_bytes {
            return Err(exceeded(path, "total payload bytes", self.total_bytes));
        }
        Ok(next)
    }
}

/// Every limit failure names the input path, resource and configured maximum.
pub(super) fn exceeded(path: &Path, resource: &str, maximum: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "packaging limit exceeded for {}: {resource} limit {maximum}",
            path.display()
        ),
    )
}
