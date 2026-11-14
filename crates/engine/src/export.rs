//! Portable, bounded inventory snapshots streamed from SQLx to JSON or JSONL.
//!
//! The versioned snapshot format is distinct from both CollectionEvent ingest
//! and graph-plan JSONL. A finalized scan is immutable through the Store API.
//! Callers request cancellation through the token and await this operation to
//! drain file writes and remove the owned partial file before returning.

mod format;
mod records;
#[cfg(test)]
mod tests;

use crate::store::{MAX_STREAM_RECORDS, Store, StoreError};
use format::ArtifactWriter;
use inventory_core::{
    CollectionFailure, CollectionScope, Edge, Resource, ScanStatus, ScanSummary, ValidationError,
    validate_identifier,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::future::Future;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_util::sync::CancellationToken;

/// Artifact discriminator shared by JSON and JSONL snapshot headers.
pub const EXPORT_ARTIFACT: &str = "aws-inventory.snapshot";
/// Portable snapshot wire version, independent of SQLite's schema version.
pub const EXPORT_VERSION: u16 = 1;
/// Maximum data records per artifact; header and summary envelopes do not count.
pub const MAX_EXPORT_RECORDS: usize = MAX_STREAM_RECORDS;
/// Maximum complete artifact size in bytes (1 TiB).
pub const MAX_EXPORT_BYTES: u64 = 1_099_511_627_776;
pub(crate) const MAX_EXPORT_RECORD_BYTES: usize = 2_359_296;
const FILE_BUFFER_BYTES: usize = 65_536;

/// Container shape for the same ordered snapshot data records.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// One JSON object containing header, records array and summary.
    Json,
    /// Newline-delimited tagged records, beginning with header and ending with summary.
    #[default]
    Jsonl,
}

/// Choose preferred values or preserve every persisted source observation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceMode {
    /// Preferred observation per resource or relationship; source/time retained.
    #[default]
    Normalized,
    /// Every persisted observation, including its collection scope and page.
    AllObservations,
}

/// Whether export derives universal `REFERENCES` edges before publication.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReferenceMode {
    /// Write only observed edges; no resolver runs.
    #[default]
    Disabled,
    /// Derive and persist universal `REFERENCES` edges from observed resource
    /// properties. Idempotent across replays.
    Derive,
}

impl ReferenceMode {
    /// The single CLI-boundary conversion from `--universal-references`.
    pub const fn from_flag(enabled: bool) -> Self {
        if enabled {
            Self::Derive
        } else {
            Self::Disabled
        }
    }

    /// Whether publication must derive and persist universal reference edges.
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Derive)
    }
}

/// Publication options; record/byte budgets are validated before creating a file.
#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    /// Container format; defaults to newline-delimited JSON.
    pub format: ExportFormat,
    /// Observation selection; defaults to one preferred value per identity.
    pub provenance: ProvenanceMode,
    /// Permit finalized partial/cancelled scans; running scans are always rejected.
    pub allow_partial: bool,
    /// Derive and persist universal `REFERENCES` edges before writing the
    /// snapshot. Default: disabled, so existing exports keep their edge set.
    pub reference_mode: ReferenceMode,
    /// Resource, edge, scope and failure records combined; excludes two envelopes.
    pub max_records: usize,
    /// All output bytes, including headers, punctuation and the final newline.
    pub max_bytes: u64,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jsonl,
            provenance: ProvenanceMode::Normalized,
            allow_partial: false,
            reference_mode: ReferenceMode::Disabled,
            max_records: MAX_EXPORT_RECORDS,
            max_bytes: MAX_EXPORT_BYTES,
        }
    }
}

impl ExportOptions {
    fn validate(self) -> Result<(), ExportError> {
        if !(1..=MAX_EXPORT_RECORDS).contains(&self.max_records) {
            return Err(ExportError::Invalid("max_records must be in 1..=10000000"));
        }
        if !(1..=MAX_EXPORT_BYTES).contains(&self.max_bytes) {
            return Err(ExportError::Invalid(
                "max_bytes must be in 1..=1099511627776",
            ));
        }
        Ok(())
    }
}

/// Export, publication and recovery failures; cleanup variants retain their cause.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Inventory reads or reference-edge persistence/restoration failed.
    #[error("inventory: {0}")]
    Store(#[from] StoreError),
    /// Direct SQLx reading of persisted export metadata or records failed.
    #[error("SQLx inventory read: {0}")]
    Sql(#[from] sqlx::Error),
    /// Output inspection, creation, writing, syncing or no-clobber linking failed.
    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Persisted JSON decoding or output serialization failed.
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// A scan identifier or persisted inventory value failed validation.
    #[error("inventory validation: {0}")]
    Validation(#[from] ValidationError),
    /// Invalid budgets, output path, scan status or persisted observation metadata.
    #[error("export validation: {0}")]
    Invalid(&'static str),
    /// A record, record count or output byte budget was exceeded.
    #[error("export bound exceeded: {0}")]
    Limit(&'static str),
    /// Cancellation was observed before the final publication link began.
    #[error("export cancelled before publication")]
    Cancelled,
    /// An earlier failure was accompanied by partial-file drain/removal failure.
    #[error("{cause}; partial-file cleanup failed (drain: {drain:?}, removal: {removal:?})")]
    Cleanup {
        /// Failure that initiated cleanup.
        #[source]
        cause: Box<Self>,
        /// Write-drain failure, or `None` if draining did not fail.
        drain: Option<std::io::Error>,
        /// Partial-file removal failure, or `None` if removal did not fail.
        removal: Option<std::io::Error>,
    },
    /// Output exists successfully, but its additional partial-file link remains.
    #[error(
        "export was published at {output:?}, but its partial-file link could not be removed: {source}"
    )]
    PublishedCleanup {
        /// Published output path; callers must not treat it as unpublished.
        output: PathBuf,
        /// Error removing the now-redundant partial-file link.
        #[source]
        source: std::io::Error,
    },
    /// Export failed and the previous synthetic reference edges could not be restored.
    #[error("{cause}; restoring the pre-export reference edges failed: {restore}")]
    Restore {
        /// Original export/publication failure.
        #[source]
        cause: Box<Self>,
        /// Failure while restoring the pre-export synthetic edge set.
        restore: Box<StoreError>,
    },
}

mod model;
pub use model::*;
mod publication;
pub use publication::*;
