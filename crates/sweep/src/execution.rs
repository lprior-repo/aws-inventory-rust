pub(crate) mod bootstrap;
use bootstrap::{bootstrap_account, discover_regions};
mod collection;
mod descriptors;
#[path = "execution/draining.rs"]
mod draining;
use draining::*;
mod slice;
use slice::SliceBudget;
pub(crate) mod discovery;
pub(crate) mod page;
pub(crate) mod routing;
mod scheduler;
#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::CredentialSource;
use aws_wire::transport::{Client, Config as TransportConfig};
use futures_util::{StreamExt, stream};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose, PlanScope, RuntimeCatalog},
    inventory::{AccountId, OperationId, Region, RegionId, ScopeKey, ServiceId},
};
use inventory_engine::{
    inventory::{InventoryStore, SweepId, WorkSpec},
    limiter::{ApiExecutor, ExecutorConfig},
};

use super::{ScopeOutcomes, SweepConfig, SweepError, SweepSummary, call, input, record};

const LEASE_MARGIN_MS: u64 = 30_000;

pub(super) struct BootstrapContext<'a, 'catalog, C: CredentialSource> {
    pub catalog: &'a RuntimeCatalog<'catalog>,
    pub client: &'a Client,
    pub executor: &'a ApiExecutor,
    pub credentials: &'a C,
    pub home: &'a RegionId,
    pub config: &'a SweepConfig,
    pub metrics: &'a crate::metrics::Accumulators,
}

pub(super) struct RuntimeContext<'a, 'catalog, C: CredentialSource, S: InventoryStore> {
    pub bootstrap: BootstrapContext<'a, 'catalog, C>,
    pub store: &'a S,
    pub account: AccountId,
    pub lease_ms: i64,
    pub endpoints: discovery::EndpointCache,
    pub consumers: page::consumer_index::ConsumerIndex<'a, 'catalog>,
    pub progress: &'a crate::control::Progress,
}

pub async fn run<S: InventoryStore>(
    catalog: &RuntimeCatalog<'_>,
    store: &S,
    config: SweepConfig,
    credentials: &impl CredentialSource,
    control: crate::SweepControl,
) -> Result<SweepSummary, SweepError> {
    let runtime = collection::Resources::new(&config, credentials)?;
    let progress = crate::control::Progress::new(control.progress);
    let collection = collection::collect(catalog, store, &config, credentials, &runtime, &progress);
    let summary = supervise(&runtime.executor, collection, control.cancel.cancelled()).await?;
    Ok(progress.finish(summary))
}

/// Bounds connections by the open-file limit; pages beyond the bound wait for a connection.
fn transport_config(
    config: &SweepConfig,
    limit: Option<descriptors::SoftLimit>,
) -> Result<TransportConfig, SweepError> {
    let max_connections = descriptors::connection_bound(config.concurrency, limit);
    if max_connections < config.concurrency {
        tracing::warn!(
            concurrency = config.concurrency,
            max_connections,
            "connection bound is below concurrency; check the open-file limit"
        );
    }
    Ok(TransportConfig {
        max_connections,
        pool_idle_per_host: max_connections.clamp(1, 8),
        extra_ca_bundle: config.ca_bundle.clone(),
        allow_plain_http: false,
        ..TransportConfig::default()
    })
}

fn executor_config(concurrency: usize) -> Result<ExecutorConfig, SweepError> {
    Ok(ExecutorConfig {
        global_concurrency: concurrency,
        per_bucket_concurrency: concurrency.clamp(1, 8),
        ..ExecutorConfig::default()
    })
}

fn lease_budget(settings: ExecutorConfig) -> Result<i64, SweepError> {
    let calls = u64::try_from(settings.global_concurrency)
        .ok()
        .and_then(|pages| pages.checked_mul(2))
        .ok_or(SweepError::Operation("executor page-call budget overflow"))?;
    let (attempt_time, backoff) = attempt_budget(settings)?;
    let total = attempt_time
        .checked_add(backoff)
        .and_then(|value| value.checked_mul(calls))
        .and_then(|value| value.checked_add(inventory_engine::limiter::MAX_RETRY_AFTER_MS))
        .and_then(|value| value.checked_add(LEASE_MARGIN_MS))
        .ok_or(SweepError::Operation("executor lease budget overflow"))?;
    i64::try_from(total)
        .map_err(|_| SweepError::Operation("executor lease budget exceeds storage range"))
}

fn attempt_budget(settings: ExecutorConfig) -> Result<(u64, u64), SweepError> {
    let attempts = u64::from(settings.max_attempts);
    let attempt_time = settings
        .attempt_timeout_ms
        .checked_add(inventory_engine::limiter::MAX_RETRY_AFTER_MS)
        .and_then(|value| value.checked_add(1_000))
        .and_then(|value| value.checked_mul(attempts))
        .ok_or(SweepError::Operation("executor attempt budget overflow"))?;
    let backoff = settings
        .max_backoff_ms
        .checked_mul(attempts.saturating_sub(1))
        .ok_or(SweepError::Operation("executor backoff budget overflow"))?;
    Ok((attempt_time, backoff))
}

async fn supervise(
    executor: &ApiExecutor,
    collection: impl Future<Output = Result<SweepSummary, SweepError>>,
    signal: impl Future<Output = ()>,
) -> Result<SweepSummary, SweepError> {
    tokio::pin!(collection);
    tokio::select! {
        biased;
        () = signal => {
            executor.cancel();
            match collection.await {
                Err(SweepError::Api(inventory_engine::limiter::ApiError::Cancelled)) | Ok(_) =>
                    Err(inventory_engine::limiter::ApiError::Cancelled.into()),
                Err(error) => Err(error),
            }
        }
        result = &mut collection => result,
    }
}

async fn sweep_id(
    store: &impl InventoryStore,
    config: &SweepConfig,
    account: &AccountId,
) -> Result<SweepId, SweepError> {
    if !config.resume {
        return SweepId::new(uuid::Uuid::new_v4().to_string()).map_err(SweepError::from);
    }
    // Discover pending sweep without mutation.
    let pending = store
        .discover_active_sweep()
        .await?
        .ok_or(SweepError::Operation("no pending sweep to resume"))?;
    // Verify manifest and claim the sweep atomically.
    store.resume_sweep(&pending, account).await?;
    Ok(pending)
}

fn endpoint_url(config: &SweepConfig, compiled: &str) -> Result<String, SweepError> {
    Ok(config.endpoint_override()?.unwrap_or(compiled).to_owned())
}

fn unique_purpose<'borrow, 'catalog>(
    catalog: &'borrow RuntimeCatalog<'catalog>,
    matches: impl Fn(&PlanPurpose) -> bool,
) -> Result<&'borrow CallablePlan<'catalog>, SweepError> {
    let mut plans = catalog
        .plans()
        .iter()
        .filter(|plan| matches(plan.purpose()));
    let plan = plans.next().ok_or(SweepError::Operation(
        "required reviewed bootstrap plan is missing",
    ))?;
    if plans.next().is_some() {
        return Err(SweepError::Operation("bootstrap purpose is ambiguous"));
    }
    Ok(plan)
}

fn parse_regions(
    rows: &[inventory_core::record::Record],
    path: &[inventory_core::record::FieldId],
) -> Result<Vec<Region>, SweepError> {
    rows.iter()
        .map(|row| {
            let value = record::path_value(row.root(), path)?
                .ok_or(SweepError::Operation("region row omitted its name"))?;
            let name = match value {
                inventory_core::record::ValueRef::Text(value) => value,
                _ => return Err(SweepError::Operation("region name is not text")),
            };
            RegionId::parse(name)
                .map(Region::Regional)
                .map_err(|_| SweepError::Operation("region name is invalid"))
        })
        .collect()
}
