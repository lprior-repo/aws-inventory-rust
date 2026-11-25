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
mod rate;
mod resolved_endpoints;
pub(crate) mod routing;
mod scheduler;
#[cfg(test)]
pub(crate) mod test_catalog;
#[cfg(test)]
mod test_transport;
#[cfg(test)]
mod tests;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_POOL_IDLE_PER_HOST: usize = 8;

pub(super) struct BootstrapContext<'a, 'catalog, C: CredentialSource> {
    pub catalog: &'a RuntimeCatalog<'catalog>,
    pub client: &'a Client,
    pub executor: &'a ApiExecutor,
    pub credentials: &'a C,
    pub home: &'a RegionId,
    pub config: &'a SweepConfig,
    pub metrics: &'a crate::metrics::Accumulators,
    pub resolved_endpoints: &'a resolved_endpoints::ResolvedEndpoints,
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
    let mut runtime = collection::Resources::new(&config, credentials)?;
    runtime.set_attempts(control.attempts);
    let progress = crate::control::Progress::new(control.progress);
    let collection = collection::collect(catalog, store, &config, credentials, &runtime, &progress);
    let summary = supervise(&runtime.executor, collection, control.cancel.cancelled()).await?;
    Ok(progress.finish(summary))
}

fn _run_is_send<'a, S, C>(
    catalog: &'a RuntimeCatalog<'a>,
    store: &'a S,
    config: SweepConfig,
    credentials: &'a C,
    control: crate::SweepControl,
) -> impl Future<Output = Result<SweepSummary, SweepError>> + Send + 'a
where
    S: InventoryStore + Sync,
    C: CredentialSource + Sync,
{
    run(catalog, store, config, credentials, control)
}

impl SweepConfig {
    pub fn validate_transport_overrides(&self) -> Result<(), SweepError> {
        let max_connections =
            descriptors::connection_bound(self.concurrency, descriptors::soft_open_files());
        pool_settings(self, max_connections, &TransportConfig::default()).map(|_| ())
    }
}

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
    let defaults = TransportConfig::default();
    let (idle_timeout, pool_idle_per_host) = pool_settings(config, max_connections, &defaults)?;
    Ok(TransportConfig {
        max_connections,
        idle_timeout,
        pool_idle_per_host,
        extra_ca_bundle: config.ca_bundle.clone(),
        allow_plain_http: false,
        ..defaults
    })
}

fn pool_settings(
    config: &SweepConfig,
    max_connections: usize,
    defaults: &TransportConfig,
) -> Result<(Duration, usize), SweepError> {
    let idle_range = defaults.idle_timeout_range();
    let idle_timeout = match config.idle_timeout {
        Some(idle) if !idle_range.contains(&idle) => {
            return Err(SweepError::InvalidIdleTimeout {
                millis: idle.as_millis(),
                min_ms: idle_range.start().as_millis(),
                max_ms: idle_range.end().as_millis(),
            });
        }
        Some(idle) => idle,
        None => DEFAULT_IDLE_TIMEOUT,
    };
    let pool_idle_per_host = match config.pool_idle_per_host {
        Some(given) if !(1..=max_connections).contains(&given) => {
            return Err(SweepError::InvalidPoolIdlePerHost {
                given,
                max: max_connections,
            });
        }
        Some(given) => given,
        None => max_connections.min(DEFAULT_POOL_IDLE_PER_HOST),
    };
    Ok((idle_timeout, pool_idle_per_host))
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
    let pending = store
        .discover_active_sweep()
        .await?
        .ok_or(SweepError::Operation("no pending sweep to resume"))?;
    store.resume_sweep(&pending, account).await?;
    Ok(pending)
}

fn endpoint_url<'a>(config: &'a SweepConfig, compiled: &'a str) -> Result<&'a str, SweepError> {
    Ok(match config.endpoint_override()? {
        Some(endpoint) => endpoint,
        None => compiled,
    })
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
