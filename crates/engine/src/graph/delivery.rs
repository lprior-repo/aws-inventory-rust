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

mod plan;
use plan::*;
mod pipeline;
pub use pipeline::*;
mod execution;
use execution::*;
