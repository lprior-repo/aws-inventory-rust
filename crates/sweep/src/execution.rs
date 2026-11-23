mod collection;
#[path = "execution/draining.rs"]
mod draining;
use draining::*;
mod slice;
use slice::SliceBudget;
pub(crate) mod discovery;
pub(crate) mod page;
pub(crate) mod routing;
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

fn transport_config(config: &SweepConfig) -> Result<TransportConfig, SweepError> {
    Ok(TransportConfig {
        max_connections: config.concurrency,
        pool_idle_per_host: config.concurrency.clamp(1, 8),
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
    // A leased batch is a barrier: no further pages enter this executor until it
    // drains. Bound its entire workload serially, including DescribeEndpoints
    // before each target call, queued permits, adaptive pacing and server hints.
    let calls = u64::try_from(settings.global_concurrency)
        .ok()
        .and_then(|pages| pages.checked_mul(2))
        .ok_or(SweepError::Operation("executor page-call budget overflow"))?;
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
    let total = attempt_time
        .checked_add(backoff)
        .and_then(|value| value.checked_mul(calls))
        // A previous batch/bootstrap call may leave an accepted server cooldown.
        .and_then(|value| value.checked_add(inventory_engine::limiter::MAX_RETRY_AFTER_MS))
        .and_then(|value| value.checked_add(LEASE_MARGIN_MS))
        .ok_or(SweepError::Operation("executor lease budget overflow"))?;
    i64::try_from(total)
        .map_err(|_| SweepError::Operation("executor lease budget exceeds storage range"))
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

async fn bootstrap_account(
    context: &BootstrapContext<'_, '_, impl CredentialSource>,
) -> Result<AccountId, SweepError> {
    let plan = unique_purpose(context.catalog, |purpose| {
        matches!(purpose, PlanPurpose::AccountIdentity { .. })
    })?;
    let PlanPurpose::AccountIdentity { account_path } = plan.purpose() else {
        return Err(SweepError::Operation(
            "account identity plan purpose changed",
        ));
    };
    let endpoint = plan.endpoint(context.home, &Region::Regional(context.home.clone()))?;
    let url = endpoint_url(context.config, &endpoint.url)?;
    let region = Region::Regional(context.home.clone());
    let base = input::root_bindings(plan, None, Some(&region))?;
    let request = call::Call {
        client: context.client,
        executor: context.executor,
        credentials: context.credentials,
        plan,
        endpoint: &endpoint,
        url: &url,
        base: &base,
        fields: &[],
        metrics: context.metrics,
    };
    let response = request.run().await?;
    let value = record::path_value(response.record.root(), account_path.as_ref())?.ok_or(
        SweepError::Operation("account identity response omitted account"),
    )?;
    let account = match value {
        inventory_core::record::ValueRef::Text(value) => value,
        _ => return Err(SweepError::Operation("account identity value is not text")),
    };
    AccountId::parse(account).map_err(SweepError::from)
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

async fn discover_regions(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
) -> Result<Vec<Region>, SweepError> {
    let requested = context
        .bootstrap
        .config
        .region
        .as_deref()
        .map(|value| {
            RegionId::parse(value).map_err(|_| SweepError::InvalidConfig("region is invalid"))
        })
        .transpose()?;
    let plan = unique_purpose(context.bootstrap.catalog, |purpose| {
        matches!(purpose, PlanPurpose::RegionDiscovery { .. })
    })?;
    let PlanPurpose::RegionDiscovery {
        regions_path,
        region_name_path,
        ..
    } = plan.purpose()
    else {
        return Err(SweepError::Operation(
            "region discovery plan purpose changed",
        ));
    };
    let endpoint = plan.endpoint(
        context.bootstrap.home,
        &Region::Regional(context.bootstrap.home.clone()),
    )?;
    let url = endpoint_url(context.bootstrap.config, &endpoint.url)?;
    let region = Region::Regional(context.bootstrap.home.clone());
    let base = input::root_bindings(plan, Some(&context.account), Some(&region))?;
    let request = call::Call {
        client: context.bootstrap.client,
        executor: context.bootstrap.executor,
        credentials: context.bootstrap.credentials,
        plan,
        endpoint: &endpoint,
        url: &url,
        base: &base,
        fields: &[],
        metrics: context.bootstrap.metrics,
    };
    let response = request.run().await?;
    let rows = record::item_records(
        &response.record,
        &inventory_core::catalog::runtime::ItemPlan::Path(regions_path.clone()),
    )?;
    let regions = parse_regions(&rows, region_name_path.as_ref())?;
    narrow_regions(regions, requested)
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
