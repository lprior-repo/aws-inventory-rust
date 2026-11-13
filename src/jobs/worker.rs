use super::*;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_engine::jobs::{Completion, JobRecord, Lease, TerminalJobState};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const MAX_CLAIMS: usize = 1024;
const MAX_HEARTBEATS: usize = 32768;
const MAX_JOB_RUNTIME: Duration = Duration::from_secs(86400);
const MAX_DRAIN_RUNTIME: Duration = Duration::from_secs(30);
const MAX_SUMMARY_BYTES: usize = 4096;
const MAX_WORKER_OUTPUT_BYTES: usize = 4_194_304;

#[tracing::instrument(skip_all)]
pub(super) async fn run(
    store: &JobStore,
    limit: usize,
    lease_seconds: u64,
    worker: Option<String>,
) -> Result<Value> {
    ensure!(
        (1..=MAX_CLAIMS).contains(&limit),
        "job limit must be 1..1024"
    );
    ensure!(
        (10..=3600).contains(&lease_seconds),
        "lease-seconds must be 10..3600"
    );
    let lease_ms = i64::try_from(lease_seconds.checked_mul(1000).context("lease overflow")?)?;
    let owner = worker.unwrap_or_else(|| format!("worker-{}", uuid::Uuid::new_v4()));
    let shutdown = CancellationToken::new();
    let work: BoxFuture<'_, Result<Value>> =
        Box::pin(run_queue(store, limit, lease_ms, &owner, &shutdown));
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal { tracing::error!(%error, "worker shutdown signal failed; stopping intake"); }
            shutdown.cancel();
            work.await
        }
    }
}

#[tracing::instrument(skip_all)]
async fn run_queue(
    store: &JobStore,
    limit: usize,
    lease_ms: i64,
    owner: &str,
    shutdown: &CancellationToken,
) -> Result<Value> {
    let mut outcomes = stream::iter(0..limit)
        .map(Ok::<_, anyhow::Error>)
        .try_fold(Outcomes::new(limit)?, |mut results, _| async move {
            if !results.accepting() {
                return Ok(results);
            }
            if shutdown.is_cancelled() {
                results.stop(StopReason::WorkerShutdown);
                return Ok(results);
            }
            let token = uuid::Uuid::new_v4().to_string();
            let Some(lease) = store
                .claim(owner, &token, demo::now_ms()?, lease_ms)
                .await?
            else {
                results.stop(StopReason::QueueEmpty);
                return Ok(results);
            };
            results.record(run_claim(store, lease, lease_ms, shutdown).await?)?;
            Ok(results)
        })
        .await?;
    if shutdown.is_cancelled() && !outcomes.cancelled_job() {
        outcomes.stop(StopReason::WorkerShutdown);
    }
    outcomes.report(owner)
}

#[cfg(test)]
mod tests;

mod summaries;
#[cfg(test)]
use inventory_engine::jobs::JobState;
use summaries::compact_result;
use summaries::completion_state;
use summaries::job_summary;
use summaries::{Cancellation, Outcomes, StopReason};

mod claim;
use claim::run_claim;

mod supervision;
use supervision::supervise;
#[cfg(test)]
use supervision::supervise_futures;
