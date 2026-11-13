//! One retry owner for bounded, replayable graph writes.
use super::{
    GraphBatch, GraphError, PlanReader, ProjectionLog, existing_resource_index, next_batch,
};
use crate::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use futures_util::{TryStreamExt, stream};
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::fs::File;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

mod replay;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PushOptions {
    pub max_attempts: u32,
    pub attempt_timeout_ms: u64,
    pub max_backoff_ms: u64,
    pub overall_timeout_ms: u64,
}

impl Default for PushOptions {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            attempt_timeout_ms: 30_000,
            max_backoff_ms: 5_000,
            overall_timeout_ms: 900_000,
        }
    }
}

impl PushOptions {
    fn validate(self) -> Result<Self, GraphError> {
        if !(1..=10).contains(&self.max_attempts)
            || !(1..=300_000).contains(&self.attempt_timeout_ms)
            || !(1..=300_000).contains(&self.max_backoff_ms)
            || !(1..=86_400_000).contains(&self.overall_timeout_ms)
        {
            return Err(GraphError::Invalid(
                "graph attempts must be1..=10, attempt/backoff1..=300000ms, overall1..=86400000ms"
                    .into(),
            ));
        }
        Ok(self)
    }

    fn executor(self, replay_safe: bool) -> Result<ApiExecutor, GraphError> {
        ApiExecutor::new(ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 100_000,
            max_attempts: if replay_safe { self.max_attempts } else { 1 },
            attempt_timeout_ms: self.attempt_timeout_ms,
            max_backoff_ms: self.max_backoff_ms,
        })
        .map_err(|_| GraphError::Invalid("invalid graph retry configuration".into()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PushSummary {
    pub applied_batches: u64,
    pub skipped_batches: u64,
    pub attempts: u64,
    pub retries: u64,
    pub reconnects: u64,
}

struct Control<'a> {
    cancellation: &'a CancellationToken,
    deadline: Instant,
}

impl Control<'_> {
    async fn run<T>(
        &self,
        operation: impl Future<Output = Result<T, GraphError>>,
    ) -> Result<T, GraphError> {
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(GraphError::Cancelled),
            () = tokio::time::sleep_until(self.deadline) => Err(GraphError::Deadline),
            result = operation => result,
        }
    }
}

#[derive(Default)]
struct FrozenPlan {
    graph: String,
    scan_id: String,
    hashes: Vec<[u8; 32]>,
    hasher: Sha256,
}

fn batch_hash(batch: &GraphBatch) -> Result<[u8; 32], GraphError> {
    Ok(Sha256::digest(serde_json::to_vec(batch)?).into())
}

fn hex(hash: &[u8]) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl FrozenPlan {
    fn digest(&self) -> String {
        format!("{:x}", self.hasher.clone().finalize())
    }

    fn hash(&self, sequence: u64) -> Result<&[u8; 32], GraphError> {
        let index = usize::try_from(sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        self.hashes.get(index).ok_or_else(|| {
            GraphError::Invalid("projection differs from the validated frozen plan".into())
        })
    }
}

async fn freeze_plan(input: &Path) -> Result<FrozenPlan, GraphError> {
    let file = File::open(input).await?;
    stream::try_unfold(PlanReader::new(file), next_batch)
        .try_fold(FrozenPlan::default(), |mut plan, batch| async move {
            let hash = batch_hash(&batch)?;
            if plan.hashes.is_empty() {
                plan.graph = batch.graph.clone();
                plan.scan_id = batch.scan_id.clone();
            }
            plan.hashes.try_reserve(1).map_err(|_| {
                GraphError::Invalid("projection fingerprint allocation failed".into())
            })?;
            plan.hashes.push(hash);
            plan.hasher.update(hash);
            Ok(plan)
        })
        .await
}

async fn bind_plan(log: &ProjectionLog, plan: &FrozenPlan) -> Result<(), GraphError> {
    sqlx::query("CREATE TABLE IF NOT EXISTS projection_plan (destination TEXT NOT NULL, graph TEXT NOT NULL, scan_id TEXT NOT NULL, digest TEXT NOT NULL, PRIMARY KEY(destination,graph,scan_id))")
        .execute(&log.pool).await?;
    let expected = plan.digest();
    let found: Option<String> = sqlx::query_scalar(
        "SELECT digest FROM projection_plan WHERE destination=?1 AND graph=?2 AND scan_id=?3",
    )
    .bind(&log.destination)
    .bind(&plan.graph)
    .bind(&plan.scan_id)
    .fetch_optional(&log.pool)
    .await?;
    if found.as_ref().is_some_and(|old| old != &expected) {
        return Err(GraphError::Invalid(
            "projection plan changed after checkpoint; use a new checkpoint or scan ID".into(),
        ));
    }
    // Also validate legacy checkpoints in full before any new server write.
    let limit = i64::try_from(super::MAX_BATCHES.saturating_add(1))
        .map_err(|_| GraphError::Invalid("checkpoint row bound overflow".into()))?;
    sqlx::query_as::<_, (i64, String)>("SELECT sequence,digest FROM projection_ack WHERE destination=?1 AND graph=?2 AND scan_id=?3 ORDER BY sequence LIMIT ?4")
        .bind(&log.destination).bind(&plan.graph).bind(&plan.scan_id).bind(limit).fetch(&log.pool)
        .map_err(GraphError::Sql)
        .try_for_each(|(sequence, old)| async move {
            let sequence = u64::try_from(sequence).map_err(|_| GraphError::Invalid("negative checkpoint sequence".into()))?;
            if old != hex(plan.hash(sequence)?) {
                return Err(GraphError::Invalid("acknowledged batch differs from the frozen plan".into()));
            }
            Ok(())
        }).await?;
    sqlx::query("INSERT INTO projection_plan(destination,graph,scan_id,digest) VALUES (?1,?2,?3,?4) ON CONFLICT(destination,graph,scan_id) DO NOTHING")
        .bind(&log.destination).bind(&plan.graph).bind(&plan.scan_id).bind(&expected).execute(&log.pool).await?;
    let bound: String = sqlx::query_scalar(
        "SELECT digest FROM projection_plan WHERE destination=?1 AND graph=?2 AND scan_id=?3",
    )
    .bind(&log.destination)
    .bind(&plan.graph)
    .bind(&plan.scan_id)
    .fetch_one(&log.pool)
    .await?;
    if bound != expected {
        return Err(GraphError::Invalid(
            "another writer bound a different graph plan".into(),
        ));
    }
    Ok(())
}

struct Projection {
    client: redis::Client,
    connection: Option<MultiplexedConnection>,
    log: ProjectionLog,
    plan: FrozenPlan,
    options: PushOptions,
    summary: PushSummary,
    connections: AtomicU64,
}

/// Validate the complete plan, bind immutable contents, then deliver serially.
/// Retries own no detached task and never wrap the SQLx acknowledgement commit.
#[tracing::instrument(skip_all, fields(input=%input.display()))]
pub async fn push_file_with_options(
    input: PathBuf,
    checkpoint: PathBuf,
    url: &str,
    options: PushOptions,
    cancellation: &CancellationToken,
) -> Result<PushSummary, GraphError> {
    let options = options.validate()?;
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(options.overall_timeout_ms))
        .ok_or_else(|| GraphError::Invalid("graph deadline overflow".into()))?;
    let control = Control {
        cancellation,
        deadline,
    };
    let plan = control.run(freeze_plan(&input)).await?;
    let client = redis::Client::open(url).map_err(|error| GraphError::Redis(redacted(&error)))?;
    let destination = format!("{:x}", Sha256::digest(url.as_bytes()));
    // Construct the pool without starting I/O so it remains owned even if the
    // first connection or schema statement is cancelled.
    let log = ProjectionLog::prepare(&checkpoint, destination);
    let mut projection = Projection {
        client,
        connection: None,
        log,
        plan,
        options,
        summary: PushSummary::default(),
        connections: AtomicU64::new(0),
    };
    let result = control
        .run(async {
            projection.log.initialize().await?;
            bind_plan(&projection.log, &projection.plan).await?;
            apply_plan(&input, &mut projection).await
        })
        .await;
    // Redis's canonical multiplexed connection aborts its owned driver when
    // its last handle drops. A server may still have applied an ambiguous write.
    drop(projection.connection.take());
    let closed = tokio::time::timeout(Duration::from_secs(10), projection.log.pool.close()).await;
    match (result, closed) {
        (_, Err(_)) => Err(GraphError::Cleanup {
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "graph checkpoint pool did not close within its drain deadline",
            ),
        }),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Ok(())) => {
            projection.summary.reconnects = projection
                .connections
                .load(Ordering::Relaxed)
                .saturating_sub(1);
            Ok(projection.summary)
        }
    }
}

async fn apply_plan(input: &Path, projection: &mut Projection) -> Result<(), GraphError> {
    let file = File::open(input).await?;
    stream::try_unfold(PlanReader::new(file), next_batch)
        .try_fold(projection, apply_batch)
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(sequence=batch.sequence, stage=%batch.stage))]
async fn apply_batch(
    projection: &mut Projection,
    batch: GraphBatch,
) -> Result<&mut Projection, GraphError> {
    let hash = batch_hash(&batch)?;
    if &hash != projection.plan.hash(batch.sequence)? {
        return Err(GraphError::Invalid(
            "projection changed after complete validation".into(),
        ));
    }
    let digest = hex(&hash);
    if projection.log.completed(&batch, &digest).await? {
        projection.summary.skipped_batches = projection.summary.skipped_batches.saturating_add(1);
        return Ok(projection);
    }
    let replay_safe = replay::is_replay_safe(&batch);
    let executor = projection.options.executor(replay_safe)?;
    let mut previous = projection.connection.take();
    let last_error = Mutex::new(None);
    let client = &projection.client;
    let connections = &projection.connections;
    let options = projection.options;
    let result = executor
        .execute("falkor-graph-write", || {
            let connection = previous.take();
            let batch = &batch;
            let last_error = &last_error;
            async move {
                send_attempt(client, connection, batch, options, connections, last_error).await
            }
        })
        .await;
    let stats = executor.stats();
    projection.summary.attempts = projection.summary.attempts.saturating_add(stats.attempts);
    projection.summary.retries = projection
        .summary
        .retries
        .saturating_add(stats.attempts.saturating_sub(1));
    match result {
        Ok(connection) => projection.connection = Some(connection),
        Err(ApiError::Exhausted(_)) => {
            return Err(GraphError::RetryExhausted {
                attempts: stats.attempts,
            });
        }
        Err(ApiError::Cancelled) => return Err(GraphError::Cancelled),
        Err(_) => {
            let error = last_error
                .into_inner()
                .map_err(|_| GraphError::Invalid("graph failure state poisoned".into()))?;
            return Err(error.map(GraphError::Redis).unwrap_or(GraphError::Timeout));
        }
    }
    // Server confirmation precedes ACK. A cancelled/failed ACK can safely replay
    // the exact same MERGE batch; no ACK is written on transport/server failure.
    projection.log.commit(&batch, &digest).await?;
    projection.summary.applied_batches = projection.summary.applied_batches.saturating_add(1);
    Ok(projection)
}

async fn send_attempt(
    client: &redis::Client,
    previous: Option<MultiplexedConnection>,
    batch: &GraphBatch,
    options: PushOptions,
    connections: &AtomicU64,
    last_error: &Mutex<Option<redis::RedisError>>,
) -> Result<MultiplexedConnection, ApiError> {
    let mut connection = match previous {
        Some(connection) => connection,
        None => {
            connections.fetch_add(1, Ordering::Relaxed);
            client
                .get_multiplexed_async_connection()
                .await
                .map_err(|error| classify(error, last_error))?
        }
    };
    let result: redis::RedisResult<redis::Value> = redis::cmd("GRAPH.QUERY")
        .arg(&batch.graph)
        .arg(&batch.query)
        .arg("timeout")
        .arg(options.attempt_timeout_ms.min(20_000))
        .arg("--compact")
        .query_async(&mut connection)
        .await;
    match result {
        Ok(_) => Ok(connection),
        Err(error) if existing_resource_index(batch, &error) => Ok(connection),
        Err(error) => Err(classify(error, last_error)),
    }
}

fn redacted(error: &redis::RedisError) -> redis::RedisError {
    (
        error.kind(),
        "FalkorDB operation failed; endpoint, credentials, query and server details omitted",
    )
        .into()
}

fn retryable(error: &redis::RedisError) -> bool {
    error.is_connection_dropped()
        || error.is_timeout()
        || matches!(
            error.kind(),
            redis::ErrorKind::Server(
                redis::ServerErrorKind::BusyLoading
                    | redis::ServerErrorKind::TryAgain
                    | redis::ServerErrorKind::ClusterDown
                    | redis::ServerErrorKind::MasterDown
            )
        )
}

fn classify(error: redis::RedisError, last_error: &Mutex<Option<redis::RedisError>>) -> ApiError {
    let transient = retryable(&error);
    if let Ok(mut slot) = last_error.lock() {
        *slot = Some(redacted(&error));
    } else {
        return ApiError::Permanent("graph failure state poisoned".into());
    }
    if transient {
        ApiError::Transient("temporary FalkorDB transport or retryable server failure".into())
    } else {
        ApiError::Permanent("FalkorDB rejected the operation".into())
    }
}
