//! Bounded, replayable FalkorDB snapshot projections with SQLx checkpoints.
use crate::store::{Store, StoreError};
use futures_util::TryStreamExt;
use inventory_core::{Edge, Resource, ScanStatus, ScanSummary};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::sync::CancellationToken;

mod delivery;
pub use delivery::{PushOptions, PushSummary, push_file_with_options};

const MAX_BATCH_ROWS: usize = 256;
const MAX_QUERY_BYTES: usize = 2_097_152;
const MAX_BATCHES: u64 = 1_000_000;
const INDEX_QUERY: &str = "CREATE INDEX FOR (r:Resource) ON (r.key)";

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("inventory: {0}")]
    Store(#[from] StoreError),
    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("projection checkpoint: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("FalkorDB transport or server rejected a command")]
    Redis(#[source] redis::RedisError),
    #[error("projection validation: {0}")]
    Invalid(String),
    #[error("projection transport timeout; replay is safe")]
    Timeout,
    #[error("graph delivery cancelled; committed checkpoints are retained")]
    Cancelled,
    #[error("graph delivery overall deadline exceeded; committed checkpoints are retained")]
    Deadline,
    #[error("graph delivery exhausted its bounded retry budget after {attempts} attempts")]
    RetryExhausted { attempts: u64 },
    #[error("graph cleanup incomplete")]
    Cleanup {
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphBatch {
    pub version: u8,
    pub graph: String,
    pub scan_id: String,
    pub sequence: u64,
    pub stage: String,
    pub rows: u64,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct GraphExportSummary {
    pub graph: String,
    pub scan_id: String,
    pub batches: u64,
    pub resources: u64,
    pub edges: u64,
}

struct BatchWriter {
    writer: BufWriter<File>,
    graph: String,
    scan_id: String,
    sequence: u64,
    rows: Vec<String>,
    bytes: usize,
}

impl BatchWriter {
    #[tracing::instrument(skip_all, fields(stage))]
    async fn emit(&mut self, stage: &str, query: String, rows: u64) -> Result<(), GraphError> {
        if query.len() > MAX_QUERY_BYTES || self.sequence >= MAX_BATCHES {
            return Err(GraphError::Invalid("graph export bound exceeded".into()));
        }
        let batch = GraphBatch {
            version: 1,
            graph: self.graph.clone(),
            scan_id: self.scan_id.clone(),
            sequence: self.sequence,
            stage: stage.into(),
            rows,
            query,
        };
        let mut bytes = serde_json::to_vec(&batch)?;
        bytes.push(b'\n');
        self.writer.write_all(&bytes).await?;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(stage))]
    async fn push(&mut self, row: String, stage: &str) -> Result<(), GraphError> {
        let next_bytes = self
            .bytes
            .checked_add(row.len())
            .ok_or_else(|| GraphError::Invalid("batch size overflow".into()))?;
        if !self.rows.is_empty()
            && (self.rows.len() >= MAX_BATCH_ROWS
                || next_bytes > MAX_QUERY_BYTES.saturating_sub(4096))
        {
            self.flush_rows(stage).await?;
        }
        if row.len() > MAX_QUERY_BYTES.saturating_sub(4096) {
            return Err(GraphError::Invalid(
                "single graph row exceeds byte limit".into(),
            ));
        }
        self.bytes = self.bytes.saturating_add(row.len());
        self.rows.push(row);
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(stage))]
    async fn flush_rows(&mut self, stage: &str) -> Result<(), GraphError> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let template = if stage == "resources" {
            RESOURCE_QUERY
        } else {
            EDGE_QUERY
        };
        let query = format!("CYPHER rows=[{}] {template}", self.rows.join(","));
        let count = u64::try_from(self.rows.len())
            .map_err(|_| GraphError::Invalid("row count overflow".into()))?;
        self.emit(stage, query, count).await?;
        self.rows.clear();
        self.bytes = 0;
        Ok(())
    }
}

const RESOURCE_QUERY: &str = "UNWIND $rows AS row MERGE (r:Resource {key:row.key}) SET r.id=row.id, r.scan_id=row.scan_id, r.kind=row.kind, r.account_id=row.account_id, r.region=row.region, r.name=row.name, r.properties_json=row.properties_json, r.tags_json=row.tags_json, r.source=row.source, r.observed_at=row.observed_at, r.unresolved=false";
const EDGE_QUERY: &str = "UNWIND $rows AS row MERGE (a:Resource {key:row.from_key}) ON CREATE SET a.id=row.from_id, a.scan_id=row.scan_id, a.unresolved=true MERGE (b:Resource {key:row.to_key}) ON CREATE SET b.id=row.to_id, b.scan_id=row.scan_id, b.unresolved=true MERGE (a)-[e:RELATES_TO {kind:row.relation, source:row.source}]->(b) SET e.observed_at=row.observed_at, e.properties_json=row.properties_json";
const MARKER_QUERY: &str =
    "MERGE (s:InventoryScan {id:$scan}) SET s.state=$state, s.resources=$resources, s.edges=$edges";

fn literal(value: &str) -> Result<String, GraphError> {
    Ok(serde_json::to_string(value)?)
}

fn resource_row(scan: &str, r: Resource) -> Result<String, GraphError> {
    let pairs = [
        ("key", format!("{scan}::{}", r.id)),
        ("id", r.id),
        ("scan_id", scan.into()),
        ("kind", r.kind),
        ("account_id", r.account_id),
        ("region", r.region),
        ("name", r.name.unwrap_or_default()),
        ("properties_json", serde_json::to_string(&r.properties)?),
        ("tags_json", serde_json::to_string(&r.tags)?),
        ("source", r.source.as_str().into()),
    ];
    row_map(&pairs, r.observed_at)
}

fn edge_row(scan: &str, e: Edge) -> Result<String, GraphError> {
    let pairs = [
        ("from_key", format!("{scan}::{}", e.from_id)),
        ("to_key", format!("{scan}::{}", e.to_id)),
        ("from_id", e.from_id),
        ("to_id", e.to_id),
        ("scan_id", scan.into()),
        ("relation", e.relation),
        ("source", e.source.as_str().into()),
        ("properties_json", serde_json::to_string(&e.properties)?),
    ];
    row_map(&pairs, e.observed_at)
}

fn row_map(pairs: &[(&str, String)], observed_at: i64) -> Result<String, GraphError> {
    let fields = pairs
        .iter()
        .map(|(key, value)| Ok(format!("{key}:{}", literal(value)?)))
        .collect::<Result<Vec<_>, GraphError>>()?;
    Ok(format!(
        "{{{},observed_at:{observed_at}}}",
        fields.join(",")
    ))
}

fn validate_name(name: &str) -> Result<(), GraphError> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(GraphError::Invalid("graph and scan names allow only ASCII letters, numbers, underscores and hyphens, up to 128 bytes".into()));
    }
    Ok(())
}

/// Export an immutable scan through bounded SQLx streams and async file writes.
#[tracing::instrument(skip(store, output))]
pub async fn export_snapshot(
    store: &Store,
    scan_id: &str,
    graph: &str,
    output: &Path,
) -> Result<GraphExportSummary, GraphError> {
    export_snapshot_with_cancellation(store, scan_id, graph, output, &CancellationToken::new())
        .await
}

/// Cancel an owned export and remove its incomplete temporary file before returning.
#[tracing::instrument(skip(store, output, cancellation))]
pub async fn export_snapshot_with_cancellation(
    store: &Store,
    scan_id: &str,
    graph: &str,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<GraphExportSummary, GraphError> {
    if cancellation.is_cancelled() {
        return Err(GraphError::Cancelled);
    }
    validate_name(graph)?;
    validate_name(scan_id)?;
    let (status, summary) = tokio::select! {
        biased;
        () = cancellation.cancelled() => return Err(GraphError::Cancelled),
        result = snapshot_header(store, scan_id) => result?,
    };
    let mut temporary = output.as_os_str().to_owned();
    temporary.push(".partial");
    let temporary = PathBuf::from(temporary);
    // Establish ownership before selecting cancellation. In particular, never
    // remove a pre-existing .partial belonging to another invocation.
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .await?;
    let mut batch = BatchWriter {
        writer: BufWriter::with_capacity(65_536, file),
        graph: graph.into(),
        scan_id: summary.scan_id.clone(),
        sequence: 0,
        rows: Vec::with_capacity(MAX_BATCH_ROWS),
        bytes: 0,
    };
    let result = tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(GraphError::Cancelled),
        result = write_snapshot(store, &summary, status, &mut batch) => result,
    };
    let result = if cancellation.is_cancelled() {
        Err(GraphError::Cancelled)
    } else {
        result
    };
    // Retain the writer outside the selected future: dropping a Tokio file
    // write future alone does not wait for its owned blocking file operation.
    let result = match tokio::time::timeout(Duration::from_secs(10), batch.writer.flush()).await {
        Ok(Ok(())) => result,
        Ok(Err(source)) => Err(GraphError::Cleanup { source }),
        Err(_) => Err(GraphError::Cleanup {
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "graph temporary writer drain deadline exceeded",
            ),
        }),
    };
    drop(batch);
    finish_export(result, &temporary, output).await
}

#[tracing::instrument(skip(store))]
async fn snapshot_header(
    store: &Store,
    scan_id: &str,
) -> Result<(ScanStatus, ScanSummary), GraphError> {
    let status = store.scan_status(scan_id).await?;
    if status == ScanStatus::Running {
        return Err(GraphError::Invalid("cannot project a running scan".into()));
    }
    Ok((status, store.summary(scan_id).await?))
}

#[tracing::instrument(skip_all)]
async fn write_snapshot(
    store: &Store,
    summary: &ScanSummary,
    status: ScanStatus,
    batch: &mut BatchWriter,
) -> Result<GraphExportSummary, GraphError> {
    begin_export(batch, summary).await?;
    let batch = export_rows(store, batch).await?;
    end_export(batch, status, summary).await
}

#[tracing::instrument(skip_all)]
async fn begin_export(batch: &mut BatchWriter, summary: &ScanSummary) -> Result<(), GraphError> {
    batch.emit("index", INDEX_QUERY.into(), 0).await?;
    batch
        .emit(
            "begin",
            scan_marker(&summary.scan_id, "loading", summary)?,
            0,
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn export_rows<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let batch = export_resources(store, batch).await?;
    batch.flush_rows("resources").await?;
    export_edges(store, batch).await
}

#[tracing::instrument(skip_all)]
async fn export_resources<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let scan = batch.scan_id.clone();
    store
        .resource_stream(&scan)
        .map_err(GraphError::Store)
        .try_fold(batch, |batch, resource| async move {
            batch
                .push(resource_row(&batch.scan_id, resource)?, "resources")
                .await?;
            Ok(batch)
        })
        .await
}

#[tracing::instrument(skip_all)]
async fn export_edges<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let scan = batch.scan_id.clone();
    store
        .edge_stream(&scan)
        .map_err(GraphError::Store)
        .try_fold(batch, |batch, edge| async move {
            batch.push(edge_row(&batch.scan_id, edge)?, "edges").await?;
            Ok(batch)
        })
        .await
}

#[tracing::instrument(skip_all)]
async fn end_export(
    batch: &mut BatchWriter,
    status: ScanStatus,
    summary: &ScanSummary,
) -> Result<GraphExportSummary, GraphError> {
    batch.flush_rows("edges").await?;
    batch
        .emit(
            "end",
            scan_marker(&summary.scan_id, status.as_str(), summary)?,
            0,
        )
        .await?;
    sync_export(batch).await?;
    Ok(GraphExportSummary {
        graph: batch.graph.clone(),
        scan_id: batch.scan_id.clone(),
        batches: batch.sequence,
        resources: summary.resources,
        edges: summary.edges,
    })
}

#[tracing::instrument(skip_all)]
async fn sync_export(batch: &mut BatchWriter) -> Result<(), GraphError> {
    batch.writer.flush().await?;
    batch.writer.get_ref().sync_all().await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn finish_export(
    result: Result<GraphExportSummary, GraphError>,
    temporary: &Path,
    output: &Path,
) -> Result<GraphExportSummary, GraphError> {
    let installed = match result {
        Ok(summary) => {
            // Atomic no-clobber installation on the same filesystem. In
            // particular, output must never replace the live inventory DB.
            tokio::fs::hard_link(temporary, output)
                .await
                .map(|()| summary)
                .map_err(GraphError::Io)
        }
        Err(error) => Err(error),
    };
    let cleanup = tokio::fs::remove_file(temporary).await;
    match cleanup {
        Ok(()) => installed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => installed,
        Err(source) => Err(GraphError::Cleanup { source }),
    }
}

fn scan_marker(scan: &str, status: &str, summary: &ScanSummary) -> Result<String, GraphError> {
    Ok(format!(
        "CYPHER scan={} state={} resources={} edges={} {MARKER_QUERY}",
        literal(scan)?,
        literal(status)?,
        summary.resources,
        summary.edges
    ))
}

struct ProjectionLog {
    pool: SqlitePool,
    destination: String,
}

impl ProjectionLog {
    fn prepare(path: &Path, destination: String) -> Self {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(options);
        Self { pool, destination }
    }

    #[tracing::instrument(skip_all)]
    async fn initialize(&self) -> Result<(), GraphError> {
        sqlx::query("CREATE TABLE IF NOT EXISTS projection_ack (destination TEXT NOT NULL, graph TEXT NOT NULL, scan_id TEXT NOT NULL, sequence INTEGER NOT NULL, digest TEXT NOT NULL, PRIMARY KEY(destination,graph,scan_id,sequence))")
            .execute(&self.pool).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn completed(&self, batch: &GraphBatch, digest: &str) -> Result<bool, GraphError> {
        let sequence = i64::try_from(batch.sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        let found: Option<String> = sqlx::query_scalar("SELECT digest FROM projection_ack WHERE destination=?1 AND graph=?2 AND scan_id=?3 AND sequence=?4")
            .bind(&self.destination).bind(&batch.graph).bind(&batch.scan_id).bind(sequence).fetch_optional(&self.pool).await?;
        match found {
            Some(old) if old == digest => Ok(true),
            Some(_) => Err(GraphError::Invalid(
                "projection batch changed after checkpoint; use a new scan ID".into(),
            )),
            None => Ok(false),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn commit(&self, batch: &GraphBatch, digest: &str) -> Result<(), GraphError> {
        let sequence = i64::try_from(batch.sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        sqlx::query("INSERT INTO projection_ack(destination,graph,scan_id,sequence,digest) VALUES (?1,?2,?3,?4,?5)")
            .bind(&self.destination).bind(&batch.graph).bind(&batch.scan_id).bind(sequence).bind(digest).execute(&self.pool).await?;
        Ok(())
    }
}

struct PlanReader {
    reader: BufReader<File>,
    identity: Option<(String, String)>,
    sequence: u64,
    stage: u8,
    ended: bool,
}

impl PlanReader {
    fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
            identity: None,
            sequence: 0,
            stage: 0,
            ended: false,
        }
    }

    fn accept(&mut self, batch: &GraphBatch) -> Result<(), GraphError> {
        validate_name(&batch.graph)?;
        validate_name(&batch.scan_id)?;
        let pair = (batch.graph.clone(), batch.scan_id.clone());
        if self.identity.as_ref().is_some_and(|old| old != &pair)
            || self.sequence != batch.sequence
            || self.ended
        {
            return Err(GraphError::Invalid(
                "mixed identities, noncontiguous sequence, or data after end marker".into(),
            ));
        }
        if batch.version != 1
            || batch.query.len() > MAX_QUERY_BYTES
            || self.sequence >= MAX_BATCHES
            || batch.rows > 256
        {
            return Err(GraphError::Invalid(
                "unsupported or oversized projection batch".into(),
            ));
        }
        let next = match batch.stage.as_str() {
            "index" => 0,
            "begin" => 1,
            "resources" => 2,
            "edges" => 3,
            "end" => 4,
            _ => return Err(GraphError::Invalid("invalid projection stage".into())),
        };
        let allowed = match self.sequence {
            0 => next == 0,
            1 => next == 1,
            _ => next >= 2 && next >= self.stage,
        };
        if !allowed {
            return Err(GraphError::Invalid("projection stages out of order".into()));
        }
        if (next < 2 || next == 4) && batch.rows != 0 {
            return Err(GraphError::Invalid(
                "metadata batch contains row count".into(),
            ));
        }
        self.identity = Some(pair);
        self.stage = next;
        self.ended = next == 4;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn next_batch(mut plan: PlanReader) -> Result<Option<(GraphBatch, PlanReader)>, GraphError> {
    let bound = u64::try_from(MAX_QUERY_BYTES.saturating_mul(2).saturating_add(8192))
        .map_err(|_| GraphError::Invalid("line bound".into()))?;
    let mut bytes = Vec::new();
    (&mut plan.reader)
        .take(bound)
        .read_until(b'\n', &mut bytes)
        .await?;
    if bytes.is_empty() {
        if !plan.ended {
            return Err(GraphError::Invalid(
                "projection is empty or lacks a final end marker".into(),
            ));
        }
        return Ok(None);
    }
    if u64::try_from(bytes.len()).map_err(|_| GraphError::Invalid("line size".into()))? >= bound {
        return Err(GraphError::Invalid("projection line exceeds limit".into()));
    }
    let batch: GraphBatch = serde_json::from_slice(&bytes)?;
    plan.accept(&batch)?;
    Ok(Some((batch, plan)))
}

fn existing_resource_index(batch: &GraphBatch, error: &redis::RedisError) -> bool {
    // FalkorDB 6 returns a server error for repeated CREATE INDEX. Only this
    // exact idempotent schema operation and exact duplicate-index response pass.
    batch.stage == "index"
        && batch.query == INDEX_QUERY
        && matches!(error.code(), Some("Attribute") | Some("ERR"))
        && matches!(
            error.detail(),
            Some("'key' is already indexed") | Some("\"Attribute\": 'key' is already indexed")
        )
}

/// Only apply trusted plans generated by this program. They contain write queries.
#[tracing::instrument(skip(url), fields(input=%input.display()))]
pub async fn push_file(input: PathBuf, checkpoint: PathBuf, url: &str) -> Result<u64, GraphError> {
    Ok(push_file_with_options(
        input,
        checkpoint,
        url,
        PushOptions::default(),
        &CancellationToken::new(),
    )
    .await?
    .applied_batches)
}

pub fn graph_batch_schema_version() -> Value {
    serde_json::json!({"version":1,"max_rows":MAX_BATCH_ROWS,"max_query_bytes":MAX_QUERY_BYTES})
}

#[cfg(test)]
mod tests {
    use super::{GraphBatch, ProjectionLog};

    #[tokio::test]
    async fn cancelled_export_removes_owned_partial_and_reports_removal_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let temporary = directory.path().join("output.partial");
        let output = directory.path().join("output.jsonl");
        tokio::fs::write(&temporary, b"incomplete").await?;
        let cancelled =
            super::finish_export(Err(super::GraphError::Cancelled), &temporary, &output).await;
        assert!(matches!(cancelled, Err(super::GraphError::Cancelled)));
        assert!(!temporary.exists());
        assert!(!output.exists());
        tokio::fs::create_dir(&temporary).await?;
        let failed =
            super::finish_export(Err(super::GraphError::Cancelled), &temporary, &output).await;
        assert!(matches!(failed, Err(super::GraphError::Cleanup { .. })));
        assert!(temporary.exists());
        Ok(())
    }

    #[tokio::test]
    async fn acknowledged_batches_are_immutable_and_destination_scoped()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("ack.sqlite");
        let batch = GraphBatch {
            version: 1,
            graph: "graph".into(),
            scan_id: "scan".into(),
            sequence: 0,
            stage: "index".into(),
            rows: 0,
            query: "CREATE INDEX FOR (r:Resource) ON (r.key)".into(),
        };
        let first = ProjectionLog::prepare(&path, "destination-one".into());
        first.initialize().await?;
        assert!(!first.completed(&batch, "digest-a").await?);
        first.commit(&batch, "digest-a").await?;
        assert!(first.completed(&batch, "digest-a").await?);
        assert!(first.completed(&batch, "digest-b").await.is_err());
        first.pool.close().await;
        let second = ProjectionLog::prepare(&path, "destination-two".into());
        second.initialize().await?;
        assert!(!second.completed(&batch, "digest-a").await?);
        second.pool.close().await;
        Ok(())
    }
}
