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

/// Stream an immutable scan, then atomically install a new output file.
///
/// Complete is required unless allow_partial explicitly permits Partial or
/// Cancelled. Running is always rejected. Existing output or `.partial` paths
/// are never overwritten. Cooperative cancellation removes only the partial
/// file created by this call. Once the final no-clobber link begins, publication
/// wins a concurrent cancellation; callers must retain the successful result.
#[tracing::instrument(skip(store, output, cancellation), fields(scan_id))]
pub async fn export_snapshot(
    store: &Store,
    scan_id: &str,
    output: &Path,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    options.validate()?;
    validate_identifier(scan_id, "scan id")?;
    check_cancel(cancellation)?;
    let temporary = partial_path(output)?;
    reject_existing(output).await?;
    let header = snapshot_header(store, scan_id, options, cancellation).await?;
    check_cancel(cancellation)?;
    let file = create_partial(&temporary).await?;
    let result = write_partial(store, file, &header, options, cancellation).await;
    publish_or_discard(result, &temporary, output, cancellation).await
}

fn partial_path(output: &Path) -> Result<PathBuf, ExportError> {
    if output.file_name().is_none() {
        return Err(ExportError::Invalid("output must name a file"));
    }
    let mut path = output.as_os_str().to_owned();
    path.push(".partial");
    Ok(PathBuf::from(path))
}

#[tracing::instrument(skip_all)]
async fn reject_existing(output: &Path) -> Result<(), ExportError> {
    match tokio::fs::symlink_metadata(output).await {
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "export output already exists",
        )
        .into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[tracing::instrument(skip_all)]
async fn create_partial(temporary: &Path) -> Result<File, ExportError> {
    let mut options = File::options();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    Ok(options.open(temporary).await?)
}

#[tracing::instrument(skip_all)]
async fn snapshot_header(
    store: &Store,
    scan_id: &str,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportHeader, ExportError> {
    let status = await_read(cancellation, store.scan_status(scan_id)).await?;
    match status {
        ScanStatus::Running => return Err(ExportError::Invalid("cannot export a running scan")),
        ScanStatus::Partial | ScanStatus::Cancelled if !options.allow_partial => {
            return Err(ExportError::Invalid(
                "partial or cancelled scan requires allow_partial",
            ));
        }
        _ => {}
    }
    let summary = await_read(cancellation, store.summary(scan_id)).await?;
    let row = await_read(
        cancellation,
        sqlx::query("SELECT started_at,finished_at FROM scans WHERE scan_id=?1")
            .bind(scan_id)
            .fetch_one(store.export_reader()),
    )
    .await?;
    Ok(ExportHeader {
        artifact: EXPORT_ARTIFACT.into(),
        version: EXPORT_VERSION,
        provenance: options.provenance,
        scan: ExportScan {
            summary,
            status,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
        },
    })
}

#[tracing::instrument(skip_all)]
async fn write_partial(
    store: &Store,
    file: File,
    header: &ExportHeader,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    let mut writer =
        ArtifactWriter::new(BufWriter::with_capacity(FILE_BUFFER_BYTES, file), options);
    let written = records::write_records(store, &mut writer, header, cancellation).await;
    // Drain outstanding file writes even after cancellation or a record error.
    let drained = writer.sink.flush().await;
    let result = combine_drain(written, drained);
    result?;
    check_cancel(cancellation)?;
    writer.sink.get_ref().sync_all().await?;
    check_cancel(cancellation)?;
    Ok(writer.summary(header))
}

fn combine_drain(
    result: Result<(), ExportError>,
    drain: Result<(), std::io::Error>,
) -> Result<(), ExportError> {
    match (result, drain) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Ok(())) => Err(error),
        (Err(cause), Err(drain)) => Err(ExportError::Cleanup {
            cause: Box::new(cause),
            drain: Some(drain),
            removal: None,
        }),
    }
}

#[tracing::instrument(skip_all)]
async fn publish_or_discard(
    result: Result<ExportSummary, ExportError>,
    temporary: &Path,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    let result = result.and_then(|summary| {
        check_cancel(cancellation)?;
        Ok(summary)
    });
    let summary = match result {
        Ok(summary) => summary,
        Err(cause) => return Err(discard_partial(temporary, cause).await),
    };
    // This commit is deliberately not raced against cancellation. Same-directory
    // hard-link installation is atomic and cannot overwrite an existing target.
    if let Err(error) = tokio::fs::hard_link(temporary, output).await {
        return Err(discard_partial(temporary, error.into()).await);
    }
    tokio::fs::remove_file(temporary)
        .await
        .map_err(|source| ExportError::PublishedCleanup {
            output: output.to_path_buf(),
            source,
        })?;
    Ok(summary)
}

#[tracing::instrument(skip_all)]
async fn discard_partial(temporary: &Path, cause: ExportError) -> ExportError {
    match tokio::fs::remove_file(temporary).await {
        Ok(()) => cause,
        Err(removal) => ExportError::Cleanup {
            cause: Box::new(cause),
            drain: None,
            removal: Some(removal),
        },
    }
}

fn check_cancel(cancellation: &CancellationToken) -> Result<(), ExportError> {
    if cancellation.is_cancelled() {
        Err(ExportError::Cancelled)
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn await_read<T, E: Into<ExportError>>(
    cancellation: &CancellationToken,
    read: impl Future<Output = Result<T, E>>,
) -> Result<T, ExportError> {
    tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(ExportError::Cancelled),
        result = read => result.map_err(Into::into),
    }
}
