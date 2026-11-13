//! Owned scan futures, bounded event routing, and one SQLx inventory writer.
use crate::{
    cli::{AccountArgs, ScanArgs},
    demo,
};
use anyhow::{Context, Result};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_aws::{
    AwsScanOptions,
    discovery::{self, AssumeRoleOptions, DiscoveryOptions, OrganizationDiscovery},
};
use inventory_core::{
    CollectionEvent, CollectionFailure, CollectionScope, ScanStatus, Selection, Source,
};
use inventory_engine::{
    limiter::{ApiExecutor, ExecutorConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

const MAX_PROFILES: usize = 128;
const PROFILE_CONCURRENCY: usize = 4;
const WRITER_QUEUE: usize = 4;
const MAX_EVENTS: u64 = 1_000_000;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct Job {
    scan_id: String,
    options: AwsScanOptions,
}
struct RoutedEvent {
    scan_id: String,
    event: CollectionEvent,
}
struct Outcome {
    scan_id: String,
    profile: Option<String>,
    target_account: Option<String>,
    error: Option<String>,
    selection: Selection,
}
struct PipelineResult {
    outcomes: Vec<Outcome>,
    cancelled: bool,
    write_error: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn run(options: ScanArgs) -> Result<Value> {
    crate::jobs::validate_scan(&options)?;
    let executor = configure_executor(&options).await?;
    run_with_executor(options, executor).await
}

/// The durable worker owns cancellation and lease renewal; all scan futures
/// remain under this call and its one bounded inventory writer.
#[tracing::instrument(skip_all)]
pub(crate) async fn run_with_executor(
    options: ScanArgs,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    crate::jobs::validate_scan(&options)?;
    crate::delivery::preflight(&options.delivery).await?;
    anyhow::ensure!(
        !executor.is_cancelled(),
        "scan cancelled before target discovery"
    );
    let preparation = prepare_jobs(&options, Arc::clone(&executor));
    tokio::pin!(preparation);
    let (jobs, discovery) = tokio::select! {
        result = &mut preparation => result?,
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal { tracing::error!(%error, "shutdown signal failed"); }
            executor.cancel();
            if let Err(error) = preparation.await {
                tracing::debug!(%error, "target discovery finished with an error during cancellation");
            }
            anyhow::bail!("scan cancelled during target discovery");
        }
    };
    anyhow::ensure!(
        !executor.is_cancelled(),
        "scan cancelled before inventory initialization"
    );
    let store = Store::open(&options.db).await?;
    let mut result = run_and_close(store, jobs, Arc::clone(&executor)).await?;
    if let Some(discovery) = discovery {
        result
            .as_object_mut()
            .context("scan result must be an object")?
            .insert("target_discovery".into(), discovery);
    }
    deliver_scan_outputs(&options, result, executor).await
}

#[tracing::instrument(skip_all)]
async fn deliver_scan_outputs(
    options: &ScanArgs,
    result: Value,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    if options.delivery.output_dir.is_none() {
        return Ok(result);
    }
    let cancellation = tokio_util::sync::CancellationToken::new();
    if executor.is_cancelled() {
        cancellation.cancel();
    }
    let work = crate::delivery::after_scan(&options.db, &options.delivery, result, &cancellation);
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        () = executor.cancelled() => {
            cancellation.cancel();
            tokio::time::timeout(std::time::Duration::from_secs(30), work).await
                .context("delivery cleanup exceeded 30 seconds after scan cancellation")?
        }
        signal = tokio::signal::ctrl_c() => {
            executor.cancel();
            cancellation.cancel();
            if let Err(error) = signal { tracing::error!(%error, "shutdown signal failed"); }
            tokio::time::timeout(std::time::Duration::from_secs(30), work).await
                .context("delivery cleanup exceeded 30 seconds after interrupt")?
        }
    }
}

#[tracing::instrument(skip_all)]
pub(crate) async fn configure_executor(options: &ScanArgs) -> Result<Arc<ApiExecutor>> {
    let config = ExecutorConfig {
        global_concurrency: options.concurrency,
        per_bucket_concurrency: options.per_bucket_concurrency,
        requests_per_second: options.rps,
        max_attempts: options.attempts,
        ..ExecutorConfig::default()
    };
    let executor = if let Some(path) = &options.rate_policy {
        let file = tokio::fs::File::open(path).await?;
        let mut bytes = Vec::new();
        file.take(1_048_577).read_to_end(&mut bytes).await?;
        anyhow::ensure!(bytes.len() <= 1_048_576, "rate policy exceeds 1 MiB");
        let policies: BTreeMap<String, inventory_engine::limiter::BucketPolicy> =
            serde_json::from_slice(&bytes)?;
        anyhow::ensure!(policies.len() <= 4096, "rate policy exceeds 4096 entries");
        ApiExecutor::with_rate_policies(config, policies)?
    } else {
        ApiExecutor::new(config)?
    };
    Ok(Arc::new(executor))
}

#[tracing::instrument(skip_all)]
async fn run_and_close(store: Store, jobs: Vec<Job>, executor: Arc<ApiExecutor>) -> Result<Value> {
    let result = run_open(&store, jobs, executor).await;
    store.close().await;
    result
}

mod targets;
use targets::prepare_jobs;
#[cfg(test)]
use targets::{job, role_options, select_accounts};

mod pipeline;
use pipeline::run_open;
#[cfg(test)]
use pipeline::{collect_jobs, startup_failure};
