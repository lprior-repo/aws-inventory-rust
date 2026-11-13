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

pub const EXPORT_ARTIFACT: &str = "aws-inventory.snapshot";
pub const EXPORT_VERSION: u16 = 1;
pub const MAX_EXPORT_RECORDS: usize = MAX_STREAM_RECORDS;
pub const MAX_EXPORT_BYTES: u64 = 1_099_511_627_776;
pub const MAX_EXPORT_RECORD_BYTES: usize = 2_359_296;
const FILE_BUFFER_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    #[default]
    Jsonl,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceMode {
    /// Preferred observation per resource or relationship; source/time retained.
    #[default]
    Normalized,
    /// Every persisted observation, including its collection scope and page.
    AllObservations,
}

#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub provenance: ProvenanceMode,
    pub allow_partial: bool,
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

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("inventory: {0}")]
    Store(#[from] StoreError),
    #[error("SQLx inventory read: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("inventory validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("export validation: {0}")]
    Invalid(&'static str),
    #[error("export bound exceeded: {0}")]
    Limit(&'static str),
    #[error("export cancelled before publication")]
    Cancelled,
    #[error("{cause}; partial-file cleanup failed (drain: {drain:?}, removal: {removal:?})")]
    Cleanup {
        #[source]
        cause: Box<Self>,
        drain: Option<std::io::Error>,
        removal: Option<std::io::Error>,
    },
    #[error(
        "export was published at {output:?}, but its partial-file link could not be removed: {source}"
    )]
    PublishedCleanup {
        output: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportHeader {
    pub artifact: String,
    pub version: u16,
    pub provenance: ProvenanceMode,
    pub scan: ExportScan,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScan {
    #[serde(flatten)]
    pub summary: ScanSummary,
    pub status: ScanStatus,
    pub started_at: i64,
    pub finished_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservationProvenance {
    pub scope: CollectionScope,
    pub page_number: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportResource {
    pub resource: Resource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportEdge {
    pub edge: Edge,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

/// Coverage evidence without opaque continuation tokens or page fingerprints.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScope {
    pub scope: CollectionScope,
    pub last_page: u32,
    pub complete: bool,
    pub failed: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportCounts {
    pub resources: u64,
    pub edges: u64,
    pub scopes: u64,
    pub failures: u64,
    /// Data records only; JSONL also has a header and a summary record.
    pub records: u64,
}

/// JSONL lines use this tagged envelope. JSON's records array contains only
/// Resource, Edge, Scope and Failure, with header/summary in top-level fields.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "record", content = "data", rename_all = "snake_case")]
pub enum ExportRecord {
    Header(ExportHeader),
    Resource(ExportResource),
    Edge(ExportEdge),
    Scope(ExportScope),
    Failure(CollectionFailure),
    Summary(ExportCounts),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportSummary {
    pub scan_id: String,
    pub status: ScanStatus,
    pub format: ExportFormat,
    pub provenance: ProvenanceMode,
    #[serde(flatten)]
    pub counts: ExportCounts,
    pub bytes: u64,
    pub sha256: String,
}

mod publication;
pub use publication::*;
