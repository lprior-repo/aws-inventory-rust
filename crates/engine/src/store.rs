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

pub const MAX_QUERY_ROWS: usize = 10_000;
pub const MAX_QUERY_BYTES: usize = 8_388_608;
pub const MAX_QUERY_COLUMNS: usize = 256;
pub const MAX_QUERY_CELLS: usize = 100_000;
pub const MAX_SQL_BYTES: usize = 65_536;
pub const MAX_PAGE_BYTES: usize = 16_777_216;
pub const MAX_RECORD_BYTES: usize = 2_097_152;
pub const MAX_EXPORT_RESOURCES: usize = 100_000;
pub const MAX_EXPORT_EDGES: usize = 250_000;
pub const MAX_EXPORT_BYTES: usize = 67_108_864;
pub const MAX_STREAM_RECORDS: usize = 10_000_000;
const MAX_FAILURE_BYTES: usize = 16_384;
// At most 1,536 resource or 1,280 edge binds; well below SQLite's 32,766 limit.
// SQL batching is internal to the atomic collection-page transaction.
const INSERT_BATCH_ROWS: usize = 128;
const QUERY_PROGRESS_CALLS: u32 = 20_000;
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
static MEMORY_DATABASE_ID: AtomicU64 = AtomicU64::new(1);

type SqlTransaction = Transaction<'static, Sqlite>;
pub type ResourceStream<'a> = Pin<Box<dyn Stream<Item = Result<Resource, StoreError>> + Send + 'a>>;
pub type EdgeStream<'a> = Pin<Box<dyn Stream<Item = Result<Edge, StoreError>> + Send + 'a>>;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("SQLx SQLite: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("inventory validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    #[error("allocation failed: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("resource bound exceeded: {0}")]
    Limit(&'static str),
    #[error("unknown scan: {0}")]
    MissingScan(String),
    #[error("unknown account: {0}")]
    MissingAccount(String),
    #[error("scan is no longer running: {0}")]
    FinishedScan(String),
    #[error("conflicting persisted state: {0}")]
    Conflict(&'static str),
    #[error("unsupported database schema version: {0}")]
    SchemaVersion(i64),
    #[error("query rejected: {0}")]
    QueryDenied(&'static str),
    #[error("store is closing")]
    Closed,
    #[error("page preparation panicked")]
    PreparationPanicked,
    #[error("page preparation was cancelled")]
    PreparationCancelled,
}

/// Persisted Cloud Control drain accounting for one scan. `attempted` and
/// `succeeded` count list scopes (one per type and region), while `unsupported`
/// counts explicit provider capability boundaries.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UniversalOutcomes {
    pub attempted: u64,
    pub succeeded: u64,
    pub unsupported: u64,
    pub failed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Checkpoint {
    pub last_page: u32,
    pub next_token: Option<String>,
    pub complete: bool,
    pub failed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GraphExportCheckpoint {
    pub scan_id: String,
    pub target: String,
    pub snapshot_id: String,
    pub resources_exported: u64,
    pub edges_exported: u64,
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
use schema::*;
mod prepared;
use prepared::*;
mod persistence;
use persistence::*;
mod scan_state;
use scan_state::*;
mod lifecycle;
use lifecycle::{FinalScanStatus, PersistedScan};
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
