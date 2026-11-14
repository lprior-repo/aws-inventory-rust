//! Durable SQLx SQLite ingestion and bounded read-only inventory queries.
//!
//! Observations, edges and the continuation checkpoint commit in one transaction.
//! Missing observations never imply deletion. SQLite work runs on SQLx's native
//! worker threads; bounded page preparation runs on a tracked blocking task.
use futures_util::{Stream, StreamExt, TryStreamExt, future, stream};
use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, ScanSummary,
    Source, ValidationError, validate_identifier,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous,
};
use sqlx::{
    Column, ConnectOptions, Connection, Row, Sqlite, SqliteConnection, SqlitePool, Transaction,
    TypeInfo, ValueRef,
};
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::Path;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::task::TaskTracker;

/// Largest row limit accepted by the read-only SQL query API.
pub const MAX_QUERY_ROWS: usize = 10_000;
pub(crate) const MAX_QUERY_BYTES: usize = 8_388_608;
pub(crate) const MAX_QUERY_COLUMNS: usize = 256;
pub(crate) const MAX_QUERY_CELLS: usize = 100_000;
pub(crate) const MAX_SQL_BYTES: usize = 65_536;
pub(crate) const MAX_PAGE_BYTES: usize = 16_777_216;
pub(crate) const MAX_RECORD_BYTES: usize = 2_097_152;
pub(crate) const MAX_EXPORT_RESOURCES: usize = 100_000;
pub(crate) const MAX_EXPORT_EDGES: usize = 250_000;
pub(crate) const MAX_EXPORT_BYTES: usize = 67_108_864;
pub(crate) const MAX_STREAM_RECORDS: usize = 10_000_000;
const MAX_FAILURE_BYTES: usize = 16_384;
// At most 1,536 resource or 1,280 edge binds; well below SQLite's 32,766 limit.
// SQL batching is internal to the atomic collection-page transaction.
const INSERT_BATCH_ROWS: usize = 128;
const QUERY_PROGRESS_CALLS: u32 = 20_000;
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
static MEMORY_DATABASE_ID: AtomicU64 = AtomicU64::new(1);

type SqlTransaction = Transaction<'static, Sqlite>;
/// Preferred scan resources in ID order; errors are yielded during polling.
pub type ResourceStream<'a> = Pin<Box<dyn Stream<Item = Result<Resource, StoreError>> + Send + 'a>>;
/// Preferred scan relationships in endpoint/relation order; errors occur while polling.
pub type EdgeStream<'a> = Pin<Box<dyn Stream<Item = Result<Edge, StoreError>> + Send + 'a>>;

/// Typed inventory input, bounded-resource, persistence and lifecycle failures.
#[derive(Debug, Error)]
pub enum StoreError {
    /// SQLite access or SQLx row conversion failed.
    #[error("SQLx SQLite: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// Persisted JSON decoding or inventory serialization failed.
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// An inventory identifier, scope, page or value failed validation.
    #[error("inventory validation: {0}")]
    Validation(#[from] ValidationError),
    /// A streaming consumer or I/O operation failed.
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    /// Fallible reservation of bounded preparation/output memory failed.
    #[error("allocation failed: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
    /// Invalid path, timestamp, lifecycle request or other input invariant.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    /// A byte, row, cell, record or checked-count bound was exceeded.
    #[error("resource bound exceeded: {0}")]
    Limit(&'static str),
    /// No scan exists with the supplied identity.
    #[error("unknown scan: {0}")]
    MissingScan(String),
    /// Account status was requested for an unregistered account.
    #[error("unknown account: {0}")]
    MissingAccount(String),
    /// A write requiring a running scan targeted a finalized scan.
    #[error("scan is no longer running: {0}")]
    FinishedScan(String),
    /// Persisted invariants, replay content, page ordering or progress conflicted.
    #[error("conflicting persisted state: {0}")]
    Conflict(&'static str),
    /// SQLite's stored schema version is not supported.
    #[error("unsupported database schema version: {0}")]
    SchemaVersion(i64),
    /// SQL syntax, effects or execution budget violated the read-only query policy.
    #[error("query rejected: {0}")]
    QueryDenied(&'static str),
    /// Shutdown closed admission for page preparation.
    #[error("store is closing")]
    Closed,
    /// A tracked blocking page-preparation task panicked.
    #[error("page preparation panicked")]
    PreparationPanicked,
    /// A tracked blocking page-preparation task was cancelled.
    #[error("page preparation was cancelled")]
    PreparationCancelled,
}

/// Persisted Cloud Control drain accounting for one scan. `attempted` and
/// `succeeded` count list scopes (one per type and region), while `unsupported`
/// counts explicit provider capability boundaries.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UniversalOutcomes {
    /// Persisted Cloud Control list-scope count, one per type and region.
    pub attempted: u64,
    /// Complete, nonfailed list scopes without an unsupported-type diagnostic.
    pub succeeded: u64,
    /// Persisted unsupported-type diagnostics joined to list scopes.
    pub unsupported: u64,
    /// Persisted list scopes whose failed flag is set.
    pub failed: u64,
}

/// Durable continuation evidence for one scan and full collection-scope identity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Checkpoint {
    /// Last committed one-based page, or zero when no page has been committed.
    pub last_page: u32,
    /// Opaque continuation token; `None` does not alone prove successful coverage.
    pub next_token: Option<String>,
    /// A terminal enumeration page was committed.
    pub complete: bool,
    /// The scope has a persisted failure that prevents successful coverage.
    pub failed: bool,
}

/// Destination/snapshot-scoped graph progress; persisted counts may only increase.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GraphExportCheckpoint {
    /// Existing finalized scan identity.
    pub scan_id: String,
    /// Caller-defined destination identity; separates independent graph targets.
    pub target: String,
    /// Caller-defined immutable snapshot identity within the destination.
    pub snapshot_id: String,
    /// Acknowledged resource count, bounded by SQLite's nonnegative `i64` range.
    pub resources_exported: u64,
    /// Acknowledged edge count, bounded by SQLite's nonnegative `i64` range.
    pub edges_exported: u64,
    /// Terminal progress marker; a completed checkpoint cannot be reopened.
    pub complete: bool,
}

/// One SQLx writer connection and one independent, query-only reader connection.
/// Call `close(self).await` after draining the callers to finish tracked work.
pub struct Store {
    writer: SqlitePool,
    reader: SqlitePool,
    preparations: TaskTracker,
    preparation_permits: Arc<Semaphore>,
}

impl Store {
    /// Internal streaming adapters share the bounded, query-only reader pool.
    pub(crate) fn export_reader(&self) -> &SqlitePool {
        &self.reader
    }

    /// Derived-edge persistence uses the single writer pool under its own
    /// `BEGIN IMMEDIATE` transaction.
    pub(crate) fn writer_pool(&self) -> &SqlitePool {
        &self.writer
    }

    /// Executes one SELECT/WITH statement on a query-only connection. File-backed
    /// databases also use SQLITE_OPEN_READONLY. Every bound fails explicitly.
    ///
    /// # Errors
    /// Rejects invalid or effectful SQL, invalid row limits, exceeded row/byte/
    /// column/cell/time bounds, unsupported cell values or database failures.
    #[tracing::instrument(skip(self, sql))]
    pub async fn query(&self, sql: &str, row_limit: usize) -> Result<Value, StoreError> {
        validate_query(sql, row_limit)?;
        let mut connection = self.reader.acquire().await?;
        install_query_progress(&mut connection).await?;
        let result = execute_query(&mut connection, sql, row_limit).await;
        reset_progress(&mut connection).await?;
        result
    }
}

mod connection;
mod graph_checkpoint;
mod scan;
mod schema;
mod streams;
pub use schema::SCHEMA_VERSION;
use schema::*;
mod prepared;
use prepared::*;
mod persistence;
use persistence::*;
mod scan_state;
use scan_state::*;
mod lifecycle;
use lifecycle::{FinalScanStatus, PersistedScan};
mod limits;
use limits::*;
mod graph_progress;
use graph_progress::*;
mod records;
use records::*;
mod query;
use query::*;
mod registry;
pub use registry::{AccountRecord, AccountStatus, CoverageRecord, CoverageState, CoverageSummary};
mod sql_policy;
use sql_policy::*;
#[cfg(test)]
mod cancellation;
