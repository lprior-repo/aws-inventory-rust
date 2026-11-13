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

    #[tracing::instrument(skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let (options, memory) = connection_options(path.as_ref())?;
        let writer = pool_options()
            .connect_with(
                options
                    .clone()
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal),
            )
            .await?;
        if let Err(error) = initialize_schema(&writer).await {
            writer.close().await;
            return Err(error);
        }
        let reader = pool_options()
            .test_before_acquire(false)
            .before_acquire(|connection, _| {
                Box::pin(async move {
                    connection.lock_handle().await?.remove_progress_handler();
                    Ok(true)
                })
            })
            .connect_with(options.read_only(!memory).pragma("query_only", "ON"))
            .await;
        match reader {
            Ok(reader) => Ok(Self {
                writer,
                reader,
                preparations: TaskTracker::new(),
                preparation_permits: Arc::new(Semaphore::new(1)),
            }),
            Err(error) => {
                writer.close().await;
                Err(error.into())
            }
        }
    }

    /// Consuming shutdown prevents simultaneous borrowers and drains preparation.
    #[tracing::instrument(skip(self))]
    pub async fn close(self) {
        self.preparation_permits.close();
        self.preparations.close();
        self.preparations.wait().await;
        self.reader.close().await;
        self.writer.close().await;
    }

    #[tracing::instrument(skip(self))]
    pub async fn begin_scan(&self, scan_id: &str, started_at: i64) -> Result<(), StoreError> {
        validate_identifier(scan_id, "scan id")?;
        if started_at < 0 {
            return Err(StoreError::InvalidInput("negative scan start"));
        }
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(
            "INSERT INTO scans(scan_id,started_at,status) VALUES (?1,?2,'running')
                     ON CONFLICT(scan_id) DO NOTHING",
        )
        .bind(scan_id)
        .bind(started_at)
        .execute(&mut *tx)
        .await?;
        let state = scan_state(&mut tx, scan_id).await?;
        if state.0 != started_at {
            return Err(StoreError::Conflict(
                "scan id already has a different start time",
            ));
        }
        if state.1 != "running" {
            return Err(StoreError::FinishedScan(scan_id.to_owned()));
        }
        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, page))]
    pub async fn apply_page(&self, scan_id: &str, page: CollectionPage) -> Result<(), StoreError> {
        validate_identifier(scan_id, "scan id")?;
        let (page, _page_permit) = self.prepare(page).await?;
        let tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        persist_page(tx, scan_id, page).await
    }

    #[tracing::instrument(skip(self, page))]
    async fn prepare(
        &self,
        page: CollectionPage,
    ) -> Result<(PreparedPage, OwnedSemaphorePermit), StoreError> {
        let permit = Arc::clone(&self.preparation_permits)
            .acquire_owned()
            .await
            .map_err(|_| StoreError::Closed)?;
        let preparation_span = tracing::Span::current();
        let task = self.preparations.spawn_blocking(move || {
            let _entered = preparation_span.enter();
            let prepared = prepare_page(page)?;
            Ok((prepared, permit))
        });
        match task.await {
            Ok(result) => result,
            Err(error) if error.is_panic() => Err(StoreError::PreparationPanicked),
            Err(_) => Err(StoreError::PreparationCancelled),
        }
    }

    #[tracing::instrument(skip(self, failure))]
    pub async fn record_failure(
        &self,
        scan_id: &str,
        failure: &CollectionFailure,
    ) -> Result<(), StoreError> {
        validate_failure(scan_id, failure)?;
        let scope_key = failure.scope.key()?;
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        require_running(&mut tx, scan_id).await?;
        insert_scope(&mut tx, scan_id, &scope_key, &failure.scope).await?;
        persist_failure(&mut tx, scan_id, &scope_key, failure).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn finish_scan(
        &self,
        scan_id: &str,
        status: ScanStatus,
        finished_at: i64,
    ) -> Result<ScanSummary, StoreError> {
        validate_identifier(scan_id, "scan id")?;
        if status == ScanStatus::Running {
            return Err(StoreError::InvalidInput("finish status cannot be running"));
        }
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        finalize_scan(&mut tx, scan_id, status, finished_at).await?;
        let summary = scan_summary(&mut tx, scan_id).await?;
        tx.commit().await?;
        Ok(summary)
    }

    pub async fn summary(&self, scan_id: &str) -> Result<ScanSummary, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        scan_summary(&mut connection, scan_id).await
    }

    pub async fn scan_status(&self, scan_id: &str) -> Result<ScanStatus, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        match scan_state(&mut connection, scan_id).await?.1.as_str() {
            "running" => Ok(ScanStatus::Running),
            "complete" => Ok(ScanStatus::Complete),
            "partial" => Ok(ScanStatus::Partial),
            "cancelled" => Ok(ScanStatus::Cancelled),
            _ => Err(StoreError::Conflict("unknown persisted scan status")),
        }
    }

    pub async fn checkpoint(
        &self,
        scan_id: &str,
        scope: &CollectionScope,
    ) -> Result<Option<Checkpoint>, StoreError> {
        validate_identifier(scan_id, "scan id")?;
        validate_scope(scope)?;
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        read_checkpoint(&mut connection, scan_id, &scope.key()?).await
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

    pub async fn resources(&self, scan_id: &str) -> Result<Vec<Resource>, StoreError> {
        collect_export(self.resource_stream(scan_id), MAX_EXPORT_RESOURCES).await
    }

    pub async fn edges(&self, scan_id: &str) -> Result<Vec<Edge>, StoreError> {
        collect_export(self.edge_stream(scan_id), MAX_EXPORT_EDGES).await
    }

    /// Deterministically ordered, one buffered row at a time; no complete snapshot allocation.
    pub fn resource_stream<'a>(&'a self, scan_id: &'a str) -> ResourceStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT resource_json FROM scan_resources WHERE scan_id=?1 ORDER BY id",
        )
    }

    pub fn edge_stream<'a>(&'a self, scan_id: &'a str) -> EdgeStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT edge_json FROM scan_edges WHERE scan_id=?1 ORDER BY from_id,to_id,relation",
        )
    }

    pub async fn visit_resources(
        &self,
        scan_id: &str,
        mut visitor: impl FnMut(Resource) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        self.resource_stream(scan_id)
            .try_for_each(|item| future::ready(visitor(item)))
            .await
    }

    pub async fn visit_edges(
        &self,
        scan_id: &str,
        mut visitor: impl FnMut(Edge) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        self.edge_stream(scan_id)
            .try_for_each(|item| future::ready(visitor(item)))
            .await
    }

    fn json_stream<'a, T>(
        &'a self,
        scan_id: &'a str,
        sql: &'static str,
    ) -> Pin<Box<dyn Stream<Item = Result<T, StoreError>> + Send + 'a>>
    where
        T: serde::de::DeserializeOwned + Send + 'a,
    {
        let records = stream::once(async move {
            self.scan_status(scan_id).await?;
            Ok::<_, StoreError>(())
        })
        .map_ok(move |_| {
            sqlx::query(sql)
                .bind(scan_id)
                .fetch(&self.reader)
                .take(MAX_STREAM_RECORDS.saturating_add(1))
                .enumerate()
                .then(|(index, row)| async move {
                    if index >= MAX_STREAM_RECORDS {
                        return Err(StoreError::Limit("stream record count"));
                    }
                    let row = row?;
                    tokio::task::coop::consume_budget().await;
                    decode_record(row)
                })
        })
        .try_flatten();
        Box::pin(records)
    }

    pub async fn graph_checkpoint(
        &self,
        scan_id: &str,
        target: &str,
        snapshot_id: &str,
    ) -> Result<Option<GraphExportCheckpoint>, StoreError> {
        validate_graph_key(scan_id, target, snapshot_id)?;
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        let row = sqlx::query(
            "SELECT scan_id,target,snapshot_id,resources_exported,edges_exported,complete
            FROM graph_export_checkpoints WHERE scan_id=?1 AND target=?2 AND snapshot_id=?3",
        )
        .bind(scan_id)
        .bind(target)
        .bind(snapshot_id)
        .fetch_optional(&mut *connection)
        .await?;
        row.as_ref().map(graph_checkpoint_from_row).transpose()
    }

    pub async fn save_graph_checkpoint(
        &self,
        checkpoint: &GraphExportCheckpoint,
    ) -> Result<(), StoreError> {
        validate_graph_key(
            &checkpoint.scan_id,
            &checkpoint.target,
            &checkpoint.snapshot_id,
        )?;
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        if scan_state(&mut tx, &checkpoint.scan_id).await?.1 == "running" {
            return Err(StoreError::InvalidInput(
                "graph export requires a finalized scan",
            ));
        }
        persist_graph_checkpoint(&mut tx, checkpoint).await?;
        tx.commit().await?;
        Ok(())
    }
}

fn pool_options() -> SqlitePoolOptions {
    SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(Duration::from_secs(5))
}

fn connection_options(path: &Path) -> Result<(SqliteConnectOptions, bool), StoreError> {
    let name = path
        .to_str()
        .ok_or(StoreError::InvalidInput("database path is not UTF-8"))?;
    if name.is_empty() || name.starts_with("file:") || name.contains('\0') {
        return Err(StoreError::InvalidInput(
            "use a filesystem path or :memory:, not a SQLite URI",
        ));
    }
    let memory = name == ":memory:";
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .command_buffer_size(8)
        .row_buffer_size(1)
        .pragma("trusted_schema", "OFF")
        .pragma("temp_store", "FILE")
        .pragma("cache_size", "-8192")
        .disable_statement_logging();
    if memory {
        let id = MEMORY_DATABASE_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| StoreError::Limit("memory database identities"))?;
        Ok((
            options
                .filename(format!(
                    "file:inventory-memory-{}-{id}?mode=memory&cache=shared",
                    std::process::id()
                ))
                .in_memory(true)
                .shared_cache(true),
            true,
        ))
    } else {
        Ok((options, false))
    }
}

async fn initialize_schema(pool: &SqlitePool) -> Result<(), StoreError> {
    let application: i64 = sqlx::query_scalar("PRAGMA application_id")
        .fetch_one(pool)
        .await?;
    if application != 0 {
        return Err(StoreError::InvalidInput(
            "database belongs to a different application",
        ));
    }
    verify_sqlite_limits(pool).await?;
    // This pool is private to Store::open until migration succeeds. On any
    // failure it is closed, so a connection with temporarily disabled foreign
    // keys cannot escape. Keep it alive on success for shared in-memory stores.
    let mut connection = pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys=OFF")
        .execute(&mut *connection)
        .await?;
    let mut tx = connection.begin_with("BEGIN IMMEDIATE").await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut *tx)
        .await?;
    if !(0..=6).contains(&version) {
        return Err(StoreError::SchemaVersion(version));
    }
    if version == 0 {
        sqlx::raw_sql(include_str!("../sql/001_inventory.sql"))
            .execute(&mut *tx)
            .await?;
    }
    if version < 2 {
        sqlx::raw_sql(include_str!("../sql/002_architecture.sql"))
            .execute(&mut *tx)
            .await?;
    }
    if version < 3 {
        migrate_sources(&mut tx).await?;
    }
    if version < 4 {
        sqlx::raw_sql(include_str!("../sql/004_global_inventory.sql"))
            .execute(&mut *tx)
            .await?;
    }
    if version < 5 {
        migrate_cloud_control_views(
            &mut tx,
            include_str!("../sql/005_cloudcontrol_expansion.sql"),
        )
        .await?;
    }
    if version < 6 {
        migrate_cloud_control_views(&mut tx, include_str!("../sql/006_cloudcontrol_parents.sql"))
            .await?;
    }
    tx.commit().await?;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&mut *connection)
        .await?;
    let foreign_keys: bool = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *connection)
        .await?;
    if !foreign_keys {
        return Err(StoreError::InvalidInput("foreign keys were not restored"));
    }
    Ok(())
}

/// Add reviewed source views without rebuilding observations or replacing user
/// schema objects. A name conflict rolls back the entire migration.
#[tracing::instrument(skip_all)]
async fn migrate_cloud_control_views(
    tx: &mut Transaction<'_, Sqlite>,
    migration: &'static str,
) -> Result<(), StoreError> {
    let statements: Vec<&'static str> = migration
        .split_terminator(';')
        .map(str::trim)
        .filter(|sql| !sql.is_empty())
        .take(514)
        .collect();
    if statements.is_empty()
        || statements.len() > 513
        || statements.iter().any(|sql| sql.len() > MAX_SQL_BYTES)
    {
        return Err(StoreError::InvalidInput(
            "invalid expanded Cloud Control views",
        ));
    }
    for statement in statements {
        sqlx::raw_sql(statement).execute(&mut **tx).await?;
    }
    Ok(())
}

/// SQLite's generalized table-rebuild migration, preserving user views,
/// indexes, and triggers. No sqlite_schema edits or legacy ALTER semantics.
async fn migrate_sources(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    verify_managed_columns(tx).await?;
    let sizes = sqlx::query(
        "SELECT count(*) AS objects, coalesce(sum(length(CAST(sql AS BLOB))),0) AS bytes,
        coalesce(max(length(CAST(sql AS BLOB))),0) AS largest
        FROM sqlite_schema WHERE sql IS NOT NULL AND (
        type IN ('view','trigger') OR (type='index' AND tbl_name IN (
            'scope_checkpoints','resource_observations','edge_observations')))",
    )
    .fetch_one(&mut **tx)
    .await?;
    if sizes.try_get::<i64, _>("objects")? > 1_024
        || sizes.try_get::<i64, _>("bytes")? > 4_194_304
        || sizes.try_get::<i64, _>("largest")? > 65_536
    {
        return Err(StoreError::Limit("migration schema objects"));
    }
    let objects = sqlx::query(
        "SELECT type, name, sql FROM sqlite_schema WHERE sql IS NOT NULL AND (
        type IN ('view','trigger') OR (type='index' AND tbl_name IN (
            'scope_checkpoints','resource_observations','edge_observations')))
        ORDER BY type DESC, name LIMIT 1024",
    )
    .fetch_all(&mut **tx)
    .await?;
    // Dropping a view also drops its INSTEAD OF triggers, so remove triggers
    // first. Restoration uses the opposite order from the schema query above.
    for (kind, keyword) in [("trigger", "TRIGGER"), ("view", "VIEW")] {
        for object in &objects {
            if object.try_get::<&str, _>("type")? != kind {
                continue;
            }
            let name: &str = object.try_get("name")?;
            // DDL identifiers cannot be bound. Only the fixed keywords above
            // and a double-quote-escaped schema identifier enter this SQL.
            let quoted = name.replace('"', "\"\"");
            let drop_sql = format!("DROP {keyword} \"{quoted}\"");
            sqlx::query(sqlx::AssertSqlSafe(drop_sql.as_str()))
                .execute(&mut **tx)
                .await?;
        }
    }
    sqlx::raw_sql(include_str!("../sql/003_sources.sql"))
        .execute(&mut **tx)
        .await?;
    for object in &objects {
        let kind: &str = object.try_get("type")?;
        let name: &str = object.try_get("name")?;
        if kind == "view" && name == "aws_ecs_service" {
            continue; // Migration 3 installs the expanded managed view.
        }
        let definition: &str = object.try_get("sql")?;
        // Replay only already-persisted, bounded CREATE definitions selected
        // from SQLite's schema. No CLI expression or bound value becomes DDL.
        sqlx::raw_sql(sqlx::AssertSqlSafe(definition))
            .execute(&mut **tx)
            .await?;
    }
    // SQLite checks the supplied SQL buffer against MAX_SQL_LENGTH, even when
    // it contains many small statements. This generator permits semicolons
    // only as statement terminators (names and JSON paths are restricted).
    let views: Vec<&'static str> = include_str!("../sql/003_cloudcontrol_views.sql")
        .split_terminator(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .take(513)
        .collect();
    if views.len() != 200 || views.iter().any(|sql| sql.len() > MAX_SQL_BYTES) {
        return Err(StoreError::InvalidInput(
            "invalid generated Cloud Control views",
        ));
    }
    for view in views {
        sqlx::raw_sql(view).execute(&mut **tx).await?;
    }
    let broken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)")
        .fetch_one(&mut **tx)
        .await?;
    if broken {
        return Err(StoreError::Conflict("migration foreign key check failed"));
    }
    Ok(())
}

async fn verify_managed_columns(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    // These tables are application-owned. Rebuilding a table with an unknown
    // extra column would silently discard its values, even if no view/index
    // refers to it. Reject extensions before any schema object is dropped.
    let expected: [(&str, &[&str]); 3] = [
        (
            "scope_checkpoints",
            &[
                "scan_id",
                "scope_key",
                "account_id",
                "region",
                "collector",
                "parent_id",
                "source",
                "last_page",
                "next_token",
                "complete",
                "failed",
            ],
        ),
        (
            "resource_observations",
            &[
                "scan_id",
                "scope_key",
                "page_number",
                "resource_id",
                "account_id",
                "region",
                "kind",
                "name",
                "source",
                "source_priority",
                "observed_at",
                "resource_json",
            ],
        ),
        (
            "edge_observations",
            &[
                "scan_id",
                "scope_key",
                "page_number",
                "from_id",
                "to_id",
                "relation",
                "source",
                "source_priority",
                "observed_at",
                "edge_json",
            ],
        ),
    ];
    for (table, names) in expected {
        let actual: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_xinfo(?) ORDER BY cid LIMIT 257")
                .bind(table)
                .fetch_all(&mut **tx)
                .await?;
        if actual.iter().map(String::as_str).ne(names.iter().copied()) {
            return Err(StoreError::InvalidInput(
                "managed table columns differ from the supported migration schema",
            ));
        }
    }
    Ok(())
}

async fn verify_sqlite_limits(pool: &SqlitePool) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT sqlite_compileoption_used('MAX_LENGTH=8388608') AS length_ok,
        sqlite_compileoption_used('MAX_SQL_LENGTH=65536') AS sql_ok,
        sqlite_compileoption_used('MAX_COLUMN=256') AS column_ok,
        sqlite_compileoption_used('MAX_EXPR_DEPTH=128') AS depth_ok",
    )
    .fetch_one(pool)
    .await?;
    for column in ["length_ok", "sql_ok", "column_ok", "depth_ok"] {
        if !row.try_get::<bool, _>(column)? {
            return Err(StoreError::InvalidInput(
                "SQLite compile bounds missing; build with repository .cargo/config.toml",
            ));
        }
    }
    Ok(())
}

struct PreparedResource {
    id: String,
    account_id: String,
    region: String,
    kind: String,
    name: Option<String>,
    source: Source,
    observed_at: i64,
    json: String,
}
struct PreparedEdge {
    from_id: String,
    to_id: String,
    relation: String,
    source: Source,
    observed_at: i64,
    json: String,
}
struct PreparedPage {
    scope: CollectionScope,
    scope_key: String,
    page_number: u32,
    next_token: Option<String>,
    fingerprint: Vec<u8>,
    resources: Vec<PreparedResource>,
    edges: Vec<PreparedEdge>,
}

fn prepare_page(page: CollectionPage) -> Result<PreparedPage, StoreError> {
    validate_page(&page)?;
    let fingerprint = page_fingerprint(&page)?;
    let scope_key = page.scope.key()?;
    let resources = page
        .resources
        .into_iter()
        .map(|resource| {
            let json = bounded_json(&resource)?;
            Ok(PreparedResource {
                id: resource.id,
                account_id: resource.account_id,
                region: resource.region,
                kind: resource.kind,
                name: resource.name,
                source: resource.source,
                observed_at: resource.observed_at,
                json,
            })
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    let edges = page
        .edges
        .into_iter()
        .map(|edge| {
            let json = bounded_json(&edge)?;
            Ok(PreparedEdge {
                from_id: edge.from_id,
                to_id: edge.to_id,
                relation: edge.relation,
                source: edge.source,
                observed_at: edge.observed_at,
                json,
            })
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(PreparedPage {
        scope: page.scope,
        scope_key,
        page_number: page.page_number,
        next_token: page.next_token,
        fingerprint,
        resources,
        edges,
    })
}

async fn persist_page(
    mut tx: SqlTransaction,
    scan_id: &str,
    page: PreparedPage,
) -> Result<(), StoreError> {
    if page_already_committed(&mut tx, scan_id, &page).await? {
        tx.commit().await?;
        return Ok(());
    }
    initialize_page(&mut tx, scan_id, &page).await?;
    let tx = insert_resources(tx, scan_id, &page).await?;
    let mut tx = insert_edges(tx, scan_id, &page).await?;
    sqlx::query("UPDATE scope_checkpoints SET last_page=?3,next_token=?4,complete=?5 WHERE scan_id=?1 AND scope_key=?2")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number))
        .bind(&page.next_token).bind(page.next_token.is_none()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn initialize_page(
    tx: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<(), StoreError> {
    require_running(tx, scan_id).await?;
    insert_scope(tx, scan_id, &page.scope_key, &page.scope).await?;
    validate_page_sequence(tx, scan_id, page).await?;
    let resources =
        i64::try_from(page.resources.len()).map_err(|_| StoreError::Limit("page resources"))?;
    let edges = i64::try_from(page.edges.len()).map_err(|_| StoreError::Limit("page edges"))?;
    sqlx::query("INSERT INTO collection_pages(scan_id,scope_key,page_number,fingerprint,next_token,resource_count,edge_count)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).bind(&page.fingerprint)
        .bind(&page.next_token).bind(resources).bind(edges).execute(tx).await?;
    Ok(())
}

async fn insert_resources(
    tx: SqlTransaction,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<SqlTransaction, StoreError> {
    stream::iter(page.resources.chunks(INSERT_BATCH_ROWS))
        .map(Ok)
        .try_fold(tx, |mut tx, rows| async move {
            if let [resource] = rows {
                insert_one_resource(&mut tx, scan_id, page, resource).await?;
            } else {
                let mut query = resource_batch(scan_id, page, rows);
                query.build().execute(&mut *tx).await?;
            }
            Ok(tx)
        })
        .await
}

async fn insert_one_resource(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
    resource: &PreparedResource,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO resource_observations(scan_id,scope_key,page_number,resource_id,account_id,
        region,kind,name,source,source_priority,observed_at,resource_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).bind(&resource.id)
        .bind(&resource.account_id).bind(&resource.region).bind(&resource.kind).bind(&resource.name)
        .bind(resource.source.as_str()).bind(i64::from(resource.source.priority()))
        .bind(resource.observed_at).bind(&resource.json).execute(connection).await?;
    Ok(())
}

fn resource_batch(
    scan_id: &str,
    page: &PreparedPage,
    rows: &[PreparedResource],
) -> sqlx::QueryBuilder<Sqlite> {
    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO resource_observations(scan_id,scope_key,page_number,resource_id,account_id,
         region,kind,name,source,source_priority,observed_at,resource_json) ",
    );
    query.push_values(rows, |mut values, resource| {
        values
            .push_bind(scan_id)
            .push_bind(&page.scope_key)
            .push_bind(i64::from(page.page_number))
            .push_bind(&resource.id)
            .push_bind(&resource.account_id)
            .push_bind(&resource.region)
            .push_bind(&resource.kind)
            .push_bind(&resource.name)
            .push_bind(resource.source.as_str())
            .push_bind(i64::from(resource.source.priority()))
            .push_bind(resource.observed_at)
            .push_bind(&resource.json);
    });
    query
}

async fn insert_edges(
    tx: SqlTransaction,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<SqlTransaction, StoreError> {
    stream::iter(page.edges.chunks(INSERT_BATCH_ROWS))
        .map(Ok)
        .try_fold(tx, |mut tx, rows| async move {
            if let [edge] = rows {
                insert_one_edge(&mut tx, scan_id, page, edge).await?;
            } else {
                let mut query = edge_batch(scan_id, page, rows);
                query.build().execute(&mut *tx).await?;
            }
            Ok(tx)
        })
        .await
}

async fn insert_one_edge(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
    edge: &PreparedEdge,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO edge_observations(scan_id,scope_key,page_number,from_id,to_id,relation,
        source,source_priority,observed_at,edge_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
    )
    .bind(scan_id)
    .bind(&page.scope_key)
    .bind(i64::from(page.page_number))
    .bind(&edge.from_id)
    .bind(&edge.to_id)
    .bind(&edge.relation)
    .bind(edge.source.as_str())
    .bind(i64::from(edge.source.priority()))
    .bind(edge.observed_at)
    .bind(&edge.json)
    .execute(connection)
    .await?;
    Ok(())
}

fn edge_batch(
    scan_id: &str,
    page: &PreparedPage,
    rows: &[PreparedEdge],
) -> sqlx::QueryBuilder<Sqlite> {
    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO edge_observations(scan_id,scope_key,page_number,from_id,to_id,relation,
         source,source_priority,observed_at,edge_json) ",
    );
    query.push_values(rows, |mut values, edge| {
        values
            .push_bind(scan_id)
            .push_bind(&page.scope_key)
            .push_bind(i64::from(page.page_number))
            .push_bind(&edge.from_id)
            .push_bind(&edge.to_id)
            .push_bind(&edge.relation)
            .push_bind(edge.source.as_str())
            .push_bind(i64::from(edge.source.priority()))
            .push_bind(edge.observed_at)
            .push_bind(&edge.json);
    });
    query
}

async fn scan_state(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<(i64, String, Option<i64>), StoreError> {
    validate_identifier(scan_id, "scan id")?;
    sqlx::query_as("SELECT started_at,status,finished_at FROM scans WHERE scan_id=?1")
        .bind(scan_id)
        .fetch_optional(connection)
        .await?
        .ok_or_else(|| StoreError::MissingScan(scan_id.to_owned()))
}

async fn require_running(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<(), StoreError> {
    if scan_state(connection, scan_id).await?.1 != "running" {
        return Err(StoreError::FinishedScan(scan_id.to_owned()));
    }
    Ok(())
}

async fn insert_scope(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    scope: &CollectionScope,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO scope_checkpoints(scan_id,scope_key,account_id,region,collector,parent_id,source)
        VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(scan_id,scope_key) DO NOTHING")
        .bind(scan_id).bind(key).bind(&scope.account_id).bind(&scope.region).bind(&scope.collector)
        .bind(&scope.parent_id).bind(scope.source.as_str()).execute(connection).await?;
    Ok(())
}

async fn read_checkpoint(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
) -> Result<Option<Checkpoint>, StoreError> {
    let row = sqlx::query("SELECT last_page,next_token,complete,failed FROM scope_checkpoints WHERE scan_id=?1 AND scope_key=?2")
        .bind(scan_id).bind(key).fetch_optional(connection).await?;
    row.as_ref()
        .map(|row| {
            Ok(Checkpoint {
                last_page: u32::try_from(row.try_get::<i64, _>("last_page")?)
                    .map_err(|_| StoreError::Conflict("invalid persisted page number"))?,
                next_token: row.try_get("next_token")?,
                complete: row.try_get("complete")?,
                failed: row.try_get("failed")?,
            })
        })
        .transpose()
}

async fn page_already_committed(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<bool, StoreError> {
    let previous: Option<Vec<u8>> = sqlx::query_scalar("SELECT fingerprint FROM collection_pages WHERE scan_id=?1 AND scope_key=?2 AND page_number=?3")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).fetch_optional(connection).await?;
    match previous {
        Some(bytes) if bytes == page.fingerprint => Ok(true),
        Some(_) => Err(StoreError::Conflict(
            "page identity reused with different content",
        )),
        None => Ok(false),
    }
}

async fn validate_page_sequence(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<(), StoreError> {
    let previous = read_checkpoint(connection, scan_id, &page.scope_key)
        .await?
        .ok_or(StoreError::Conflict("scope checkpoint missing"))?;
    let expected = previous
        .last_page
        .checked_add(1)
        .ok_or(StoreError::Limit("page sequence overflow"))?;
    if previous.complete || page.page_number != expected {
        return Err(StoreError::Conflict(
            "page is out of order or scope is complete",
        ));
    }
    if let Some(token) = &page.next_token {
        let repeated: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM collection_pages WHERE scan_id=?1 AND scope_key=?2 AND next_token=?3)")
            .bind(scan_id).bind(&page.scope_key).bind(token).fetch_one(connection).await?;
        if repeated {
            return Err(StoreError::Conflict("continuation token cycle"));
        }
    }
    Ok(())
}

fn validate_failure(scan_id: &str, failure: &CollectionFailure) -> Result<(), StoreError> {
    validate_identifier(scan_id, "scan id")?;
    validate_scope(&failure.scope)?;
    validate_identifier(&failure.category, "failure category")?;
    if failure.message.len() > MAX_FAILURE_BYTES {
        return Err(StoreError::Limit("failure message bytes"));
    }
    Ok(())
}

async fn persist_failure(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    failure: &CollectionFailure,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO collection_failures(scan_id,scope_key,category,message) VALUES (?1,?2,?3,?4) ON CONFLICT DO NOTHING")
        .bind(scan_id).bind(key).bind(&failure.category).bind(&failure.message).execute(&mut *connection).await?;
    sqlx::query("UPDATE scope_checkpoints SET failed=1 WHERE scan_id=?1 AND scope_key=?2")
        .bind(scan_id)
        .bind(key)
        .execute(connection)
        .await?;
    Ok(())
}

async fn finalize_scan(
    connection: &mut SqliteConnection,
    scan_id: &str,
    status: ScanStatus,
    finished_at: i64,
) -> Result<(), StoreError> {
    let state = scan_state(connection, scan_id).await?;
    if finished_at < state.0 {
        return Err(StoreError::InvalidInput("scan finish precedes start"));
    }
    let effective = effective_status(connection, scan_id, status).await?;
    if state.1 != "running" {
        if state.1 != effective.as_str() || state.2 != Some(finished_at) {
            return Err(StoreError::Conflict(
                "scan was already finalized differently",
            ));
        }
        return Ok(());
    }
    sqlx::query("UPDATE scans SET status=?2,finished_at=?3 WHERE scan_id=?1")
        .bind(scan_id)
        .bind(effective.as_str())
        .bind(finished_at)
        .execute(connection)
        .await?;
    Ok(())
}

async fn effective_status(
    connection: &mut SqliteConnection,
    scan_id: &str,
    requested: ScanStatus,
) -> Result<ScanStatus, StoreError> {
    if requested != ScanStatus::Complete {
        return Ok(requested);
    }
    let (total, incomplete): (i64, i64) = sqlx::query_as(
        "SELECT count(*),coalesce(sum(CASE WHEN complete=0 OR failed=1 THEN 1 ELSE 0 END),0)
        FROM scope_checkpoints WHERE scan_id=?1",
    )
    .bind(scan_id)
    .fetch_one(connection)
    .await?;
    if total == 0 || incomplete > 0 {
        Ok(ScanStatus::Partial)
    } else {
        Ok(ScanStatus::Complete)
    }
}

async fn scan_summary(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<ScanSummary, StoreError> {
    scan_state(connection, scan_id).await?;
    let row = sqlx::query("SELECT
        (SELECT count(*) FROM scan_resources WHERE scan_id=?1) AS resources,
        (SELECT count(*) FROM scan_edges WHERE scan_id=?1) AS edges,
        (SELECT count(*) FROM scope_checkpoints WHERE scan_id=?1 AND complete=1 AND failed=0) AS completed,
        (SELECT count(*) FROM scope_checkpoints WHERE scan_id=?1 AND failed=1) AS failed")
        .bind(scan_id).fetch_one(connection).await?;
    Ok(ScanSummary {
        scan_id: scan_id.to_owned(),
        resources: row_u64(&row, "resources")?,
        edges: row_u64(&row, "edges")?,
        completed_scopes: row_u64(&row, "completed")?,
        failed_scopes: row_u64(&row, "failed")?,
    })
}

fn validate_graph_key(scan_id: &str, target: &str, snapshot_id: &str) -> Result<(), StoreError> {
    validate_identifier(scan_id, "scan id")?;
    validate_identifier(target, "graph target")?;
    validate_identifier(snapshot_id, "graph snapshot id")?;
    Ok(())
}

fn graph_checkpoint_from_row(row: &SqliteRow) -> Result<GraphExportCheckpoint, StoreError> {
    Ok(GraphExportCheckpoint {
        scan_id: row.try_get("scan_id")?,
        target: row.try_get("target")?,
        snapshot_id: row.try_get("snapshot_id")?,
        resources_exported: row_u64(row, "resources_exported")?,
        edges_exported: row_u64(row, "edges_exported")?,
        complete: row.try_get("complete")?,
    })
}

async fn persist_graph_checkpoint(
    connection: &mut SqliteConnection,
    checkpoint: &GraphExportCheckpoint,
) -> Result<(), StoreError> {
    let resources = i64::try_from(checkpoint.resources_exported)
        .map_err(|_| StoreError::Limit("graph resource count"))?;
    let edges = i64::try_from(checkpoint.edges_exported)
        .map_err(|_| StoreError::Limit("graph edge count"))?;
    let changed = sqlx::query("INSERT INTO graph_export_checkpoints
        (scan_id,target,snapshot_id,resources_exported,edges_exported,complete) VALUES (?1,?2,?3,?4,?5,?6)
        ON CONFLICT(scan_id,target,snapshot_id) DO UPDATE SET resources_exported=excluded.resources_exported,
          edges_exported=excluded.edges_exported,complete=excluded.complete
        WHERE excluded.resources_exported >= resources_exported AND excluded.edges_exported >= edges_exported
          AND excluded.complete >= complete")
        .bind(&checkpoint.scan_id).bind(&checkpoint.target).bind(&checkpoint.snapshot_id)
        .bind(resources).bind(edges).bind(checkpoint.complete).execute(connection).await?.rows_affected();
    if changed != 1 {
        return Err(StoreError::Conflict(
            "graph checkpoint would move backwards",
        ));
    }
    Ok(())
}

fn row_u64(row: &SqliteRow, column: &str) -> Result<u64, StoreError> {
    u64::try_from(row.try_get::<i64, _>(column)?)
        .map_err(|_| StoreError::Conflict("negative persisted count"))
}

fn decode_record<T: serde::de::DeserializeOwned>(row: SqliteRow) -> Result<T, StoreError> {
    let json: &str = row.try_get(0)?;
    if json.len() > MAX_RECORD_BYTES {
        return Err(StoreError::Limit("stored record bytes"));
    }
    Ok(serde_json::from_str(json)?)
}

async fn collect_export<T: Serialize>(
    records: impl Stream<Item = Result<T, StoreError>>,
    limit: usize,
) -> Result<Vec<T>, StoreError> {
    let (result, _) = records
        .try_fold(
            (Vec::new(), ByteBudget::new(MAX_EXPORT_BYTES)),
            |(mut result, mut budget), value| {
                let operation = (|| {
                    charge_export(&mut budget, &value, result.len(), limit)?;
                    result.try_reserve(1)?;
                    result.push(value);
                    Ok((result, budget))
                })();
                future::ready(operation)
            },
        )
        .await?;
    Ok(result)
}

async fn install_query_progress(connection: &mut SqliteConnection) -> Result<(), StoreError> {
    let started = Instant::now();
    let mut remaining = QUERY_PROGRESS_CALLS;
    connection
        .lock_handle()
        .await?
        .set_progress_handler(1_000, move || {
            remaining = remaining.saturating_sub(1);
            remaining != 0 && started.elapsed() <= QUERY_TIMEOUT
        });
    Ok(())
}

async fn reset_progress(connection: &mut SqliteConnection) -> Result<(), StoreError> {
    connection.lock_handle().await?.remove_progress_handler();
    Ok(())
}

struct QueryOutput {
    rows: Vec<Value>,
    columns: Option<Vec<String>>,
    budget: ByteBudget,
    cells: usize,
}

impl QueryOutput {
    fn new(row_limit: usize) -> Result<Self, StoreError> {
        let mut rows = Vec::new();
        rows.try_reserve(row_limit)?;
        let mut budget = ByteBudget::new(MAX_QUERY_BYTES);
        budget.add(2)?;
        Ok(Self {
            rows,
            columns: None,
            budget,
            cells: 0,
        })
    }
    fn push(mut self, row: SqliteRow, row_limit: usize) -> Result<Self, StoreError> {
        if self.rows.len() >= row_limit {
            return Err(StoreError::Limit(
                "query rows; narrow the query or use streaming export",
            ));
        }
        if self.columns.is_none() {
            self.columns = Some(query_columns(&row)?);
        }
        let columns = self
            .columns
            .as_ref()
            .ok_or(StoreError::Conflict("query columns unavailable"))?;
        self.cells = self
            .cells
            .checked_add(columns.len())
            .ok_or(StoreError::Limit("query cells"))?;
        if self.cells > MAX_QUERY_CELLS {
            return Err(StoreError::Limit("query cells"));
        }
        self.budget.add(3)?;
        self.rows.push(query_row(&row, columns, &mut self.budget)?);
        Ok(self)
    }
}

async fn execute_query(
    connection: &mut SqliteConnection,
    sql: &str,
    row_limit: usize,
) -> Result<Value, StoreError> {
    // Intentional user-authored SQL, already validated as one SELECT/WITH statement;
    // isolation and mutation prevention are enforced by the independent reader.
    let query = sqlx::query(sqlx::AssertSqlSafe(sql));
    let result = query
        .fetch(connection)
        .map_err(StoreError::from)
        .take(row_limit.saturating_add(1))
        .try_fold(QueryOutput::new(row_limit)?, |output, row| async move {
            tokio::task::coop::consume_budget().await;
            output.push(row, row_limit)
        })
        .await?;
    Ok(Value::Array(result.rows))
}

fn query_columns(row: &SqliteRow) -> Result<Vec<String>, StoreError> {
    if row.columns().len() > MAX_QUERY_COLUMNS {
        return Err(StoreError::Limit("query columns"));
    }
    let mut columns = Vec::new();
    let mut distinct = BTreeSet::new();
    columns.try_reserve(row.columns().len())?;
    for column in row.columns() {
        let name = column.name();
        if !distinct.insert(name) {
            return Err(StoreError::QueryDenied(
                "duplicate column names; assign distinct aliases",
            ));
        }
        columns.push(name.to_owned());
    }
    Ok(columns)
}

fn query_row(
    row: &SqliteRow,
    columns: &[String],
    budget: &mut ByteBudget,
) -> Result<Value, StoreError> {
    let mut object = Map::new();
    for (index, name) in columns.iter().enumerate() {
        budget.charge(name)?;
        budget.add(2)?;
        let cell = sql_value(row, index, budget)?;
        object.insert(name.clone(), cell);
    }
    Ok(Value::Object(object))
}

fn sql_value(row: &SqliteRow, index: usize, budget: &mut ByteBudget) -> Result<Value, StoreError> {
    let raw = row.try_get_raw(index)?;
    let value = if raw.is_null() {
        Value::Null
    } else {
        match raw.type_info().name() {
            "INTEGER" | "BOOLEAN" => Value::Number(row.try_get::<i64, _>(index)?.into()),
            "REAL" => Value::Number(
                Number::from_f64(row.try_get(index)?)
                    .ok_or(StoreError::QueryDenied("non-finite numeric result"))?,
            ),
            "TEXT" | "DATE" | "TIME" | "DATETIME" => {
                let text: &str = row.try_get(index)?;
                if text.len() > budget.limit.saturating_sub(budget.used) {
                    return Err(StoreError::Limit("query bytes"));
                }
                Value::String(text.to_owned())
            }
            "BLOB" => {
                return Err(StoreError::QueryDenied(
                    "BLOB output requires an explicit hex() projection",
                ));
            }
            _ => return Err(StoreError::QueryDenied("unsupported SQLite result type")),
        }
    };
    budget.charge(&value)?;
    Ok(value)
}

fn validate_scope(scope: &CollectionScope) -> Result<(), StoreError> {
    validate_identifier(&scope.account_id, "scope account")?;
    validate_identifier(&scope.region, "scope region")?;
    validate_identifier(&scope.collector, "scope collector")?;
    if let Some(parent) = &scope.parent_id {
        validate_identifier(parent, "scope parent")?;
    }
    Ok(())
}

fn validate_page(page: &CollectionPage) -> Result<(), StoreError> {
    page.validate()?;
    for resource in &page.resources {
        if resource.source != page.scope.source {
            return Err(StoreError::InvalidInput(
                "resource provenance differs from scope",
            ));
        }
    }
    for edge in &page.edges {
        if edge.source != page.scope.source {
            return Err(StoreError::InvalidInput(
                "edge provenance differs from scope",
            ));
        }
    }
    Ok(())
}

fn charge_export<T: Serialize>(
    budget: &mut ByteBudget,
    value: &T,
    count: usize,
    limit: usize,
) -> Result<(), StoreError> {
    if count >= limit {
        return Err(StoreError::Limit("export row count; use streaming export"));
    }
    budget
        .charge(value)
        .map_err(|_| StoreError::Limit("export bytes; use streaming export"))
}

struct ByteBudget {
    used: usize,
    limit: usize,
}

impl ByteBudget {
    fn new(limit: usize) -> Self {
        Self { used: 0, limit }
    }

    fn charge<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), StoreError> {
        serde_json::to_writer(self, value).map_err(|_| StoreError::Limit("serialized output bytes"))
    }

    fn add(&mut self, size: usize) -> io::Result<()> {
        let next = self
            .used
            .checked_add(size)
            .ok_or_else(|| io::Error::other("byte count overflow"))?;
        if next > self.limit {
            return Err(io::Error::other("serialized byte limit exceeded"));
        }
        self.used = next;
        Ok(())
    }
}

impl Write for ByteBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.add(bytes.len())?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct DigestWriter {
    budget: ByteBudget,
    digest: Sha256,
}

impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.budget.add(bytes.len())?;
        self.digest.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn page_fingerprint(page: &CollectionPage) -> Result<Vec<u8>, StoreError> {
    let mut writer = DigestWriter {
        budget: ByteBudget::new(MAX_PAGE_BYTES),
        digest: Sha256::new(),
    };
    serde_json::to_writer(&mut writer, page).map_err(|_| StoreError::Limit("page bytes"))?;
    Ok(writer.digest.finalize().to_vec())
}

struct RecordWriter {
    budget: ByteBudget,
    bytes: Vec<u8>,
}

impl Write for RecordWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.budget.add(bytes.len())?;
        self.bytes
            .try_reserve(bytes.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn bounded_json<T: Serialize>(value: &T) -> Result<String, StoreError> {
    let mut writer = RecordWriter {
        budget: ByteBudget::new(MAX_RECORD_BYTES),
        bytes: Vec::new(),
    };
    serde_json::to_writer(&mut writer, value)?;
    String::from_utf8(writer.bytes)
        .map_err(|_| StoreError::InvalidInput("serialized JSON is not UTF-8"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SqlLexState {
    Normal,
    Single,
    Double,
    Backtick,
    Bracket,
    LineComment,
    BlockComment,
}

fn validate_query(sql: &str, row_limit: usize) -> Result<(), StoreError> {
    if row_limit == 0 || row_limit > MAX_QUERY_ROWS {
        return Err(StoreError::Limit("query row limit"));
    }
    if sql.is_empty() || sql.len() > MAX_SQL_BYTES || sql.contains('\0') {
        return Err(StoreError::QueryDenied(
            "empty, oversized, or NUL-containing SQL",
        ));
    }
    validate_query_prefix(sql)?;
    let mut state = SqlLexState::Normal;
    let mut ended = false;
    let mut skip = false;
    for (index, byte) in sql.bytes().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        let next = index
            .checked_add(1)
            .and_then(|position| sql.as_bytes().get(position))
            .copied();
        match state {
            SqlLexState::Normal => lex_normal(byte, next, &mut state, &mut ended, &mut skip)?,
            SqlLexState::LineComment if byte == b'\n' || byte == b'\r' => {
                state = SqlLexState::Normal
            }
            SqlLexState::BlockComment if byte == b'*' && next == Some(b'/') => {
                state = SqlLexState::Normal;
                skip = true;
            }
            SqlLexState::Bracket if byte == b']' => state = SqlLexState::Normal,
            SqlLexState::Single if byte == b'\'' => close_quote(next, byte, &mut state, &mut skip),
            SqlLexState::Double if byte == b'"' => close_quote(next, byte, &mut state, &mut skip),
            SqlLexState::Backtick if byte == b'`' => close_quote(next, byte, &mut state, &mut skip),
            _ => {}
        }
    }
    if !matches!(state, SqlLexState::Normal | SqlLexState::LineComment) {
        return Err(StoreError::QueryDenied(
            "unterminated SQL literal or comment",
        ));
    }
    Ok(())
}

fn close_quote(next: Option<u8>, quote: u8, state: &mut SqlLexState, skip: &mut bool) {
    if next == Some(quote) {
        *skip = true;
    } else {
        *state = SqlLexState::Normal;
    }
}

fn lex_normal(
    byte: u8,
    next: Option<u8>,
    state: &mut SqlLexState,
    ended: &mut bool,
    skip: &mut bool,
) -> Result<(), StoreError> {
    if byte.is_ascii_whitespace() {
        return Ok(());
    }
    if byte == b'-' && next == Some(b'-') {
        *state = SqlLexState::LineComment;
        *skip = true;
        return Ok(());
    }
    if byte == b'/' && next == Some(b'*') {
        *state = SqlLexState::BlockComment;
        *skip = true;
        return Ok(());
    }
    if *ended {
        return Err(StoreError::QueryDenied(
            "multiple SQL statements are forbidden",
        ));
    }
    match byte {
        b';' => *ended = true,
        b'\'' => *state = SqlLexState::Single,
        b'"' => *state = SqlLexState::Double,
        b'`' => *state = SqlLexState::Backtick,
        b'[' => *state = SqlLexState::Bracket,
        _ => {}
    }
    Ok(())
}

fn validate_query_prefix(sql: &str) -> Result<(), StoreError> {
    let mut state = SqlLexState::Normal;
    let mut skip = false;
    for (index, byte) in sql.bytes().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        let next = index
            .checked_add(1)
            .and_then(|position| sql.as_bytes().get(position))
            .copied();
        match state {
            SqlLexState::LineComment => {
                if byte == b'\n' || byte == b'\r' {
                    state = SqlLexState::Normal;
                }
            }
            SqlLexState::BlockComment => {
                if byte == b'*' && next == Some(b'/') {
                    state = SqlLexState::Normal;
                    skip = true;
                }
            }
            _ if byte.is_ascii_whitespace() => {}
            _ if byte == b'-' && next == Some(b'-') => {
                state = SqlLexState::LineComment;
                skip = true;
            }
            _ if byte == b'/' && next == Some(b'*') => {
                state = SqlLexState::BlockComment;
                skip = true;
            }
            _ => {
                let tail = sql
                    .get(index..)
                    .ok_or(StoreError::QueryDenied("invalid SQL prefix"))?;
                let keyword = tail
                    .split(|ch: char| !ch.is_ascii_alphabetic())
                    .next()
                    .ok_or(StoreError::QueryDenied("missing SQL keyword"))?;
                if keyword.eq_ignore_ascii_case("select") || keyword.eq_ignore_ascii_case("with") {
                    return Ok(());
                }
                return Err(StoreError::QueryDenied(
                    "only SELECT or WITH queries are permitted",
                ));
            }
        }
    }
    Err(StoreError::QueryDenied("missing SELECT or WITH query"))
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::collections::BTreeMap;
    use tokio::sync::Notify;

    #[tokio::test]
    async fn cancelling_after_first_sqlite_observation_rolls_back_whole_page()
    -> Result<(), StoreError> {
        let store = Store::open(":memory:").await?;
        store.begin_scan("cancel", 1).await?;
        let first_write = Arc::new(Notify::new());
        let notification = Arc::clone(&first_write);
        {
            let mut writer = store.writer.acquire().await?;
            writer.lock_handle().await?.set_update_hook(move |event| {
                if event.table == "resource_observations" {
                    notification.notify_one();
                }
            });
        }
        let scope = CollectionScope {
            account_id: "111111111111".to_owned(),
            region: "us-east-1".to_owned(),
            collector: "ec2".to_owned(),
            parent_id: None,
            source: Source::Direct,
        };
        let resources = (0..1_000_u32)
            .map(|index| Resource {
                id: format!("i-{index}"),
                account_id: scope.account_id.clone(),
                region: scope.region.clone(),
                kind: "aws_ec2_instance".to_owned(),
                name: None,
                properties: serde_json::json!({}),
                tags: BTreeMap::new(),
                source: Source::Direct,
                observed_at: 10,
            })
            .collect();
        let page = CollectionPage {
            scope: scope.clone(),
            page_number: 1,
            next_token: Some("continue".to_owned()),
            resources,
            edges: Vec::new(),
        };
        {
            let ingestion = store.apply_page("cancel", page);
            tokio::pin!(ingestion);
            tokio::select! {
                biased;
                () = first_write.notified() => {}
                result = &mut ingestion => {
                    result?;
                    return Err(StoreError::Conflict("page completed before cancellation injection"));
                }
            }
        }
        // Reacquisition waits for SQLx's queued transaction rollback.
        {
            let mut writer = store.writer.acquire().await?;
            writer.lock_handle().await?.remove_update_hook();
        }
        assert_eq!(store.summary("cancel").await?.resources, 0);
        assert!(store.checkpoint("cancel", &scope).await?.is_none());
        assert_eq!(
            store
                .query("SELECT count(*) AS n FROM collection_pages", 1)
                .await?,
            serde_json::json!([{"n":0}])
        );
        store
            .apply_page(
                "cancel",
                CollectionPage {
                    scope,
                    page_number: 1,
                    next_token: None,
                    resources: Vec::new(),
                    edges: Vec::new(),
                },
            )
            .await?;
        assert_eq!(
            store
                .finish_scan("cancel", ScanStatus::Complete, 20)
                .await?
                .completed_scopes,
            1
        );
        store.close().await;
        Ok(())
    }
}
