//! Retry-safe batch submission and explicit attempt classification.
use super::*;

#[tracing::instrument(skip_all, fields(sequence=batch.sequence, stage=%batch.stage))]
pub(super) async fn apply_batch(
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

pub(super) async fn send_attempt(
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

pub(super) fn redacted(error: &redis::RedisError) -> redis::RedisError {
    (
        error.kind(),
        "FalkorDB operation failed; endpoint, credentials, query and server details omitted",
    )
        .into()
}

pub(super) fn retryable(error: &redis::RedisError) -> bool {
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

pub(super) fn classify(
    error: redis::RedisError,
    last_error: &Mutex<Option<redis::RedisError>>,
) -> ApiError {
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
