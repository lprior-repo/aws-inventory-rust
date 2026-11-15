#[path = "execution/draining.rs"]
mod draining;
use draining::*;
mod page;
#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use aws_wire::transport::{Client, Config as TransportConfig};
use futures_util::{StreamExt, stream};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose, PlanScope, RuntimeCatalog},
    inventory::{AccountId, OperationId, Region, RegionId, ScopeKey, ServiceId},
};
use inventory_engine::{
    inventory::{BinaryStore, SweepId, WorkSpec},
    limiter::{ApiExecutor, ExecutorConfig},
};

use super::{ScopeOutcomes, SweepConfig, SweepError, SweepSummary, call, input, record};

const LEASE_MARGIN_MS: u64 = 30_000;

pub(super) struct BootstrapContext<'a, 'catalog> {
    pub catalog: &'a RuntimeCatalog<'catalog>,
    pub client: &'a Client,
    pub executor: &'a ApiExecutor,
    pub credentials: &'a ProfileCredentials,
    pub home: &'a RegionId,
    pub config: &'a SweepConfig,
}

pub(super) struct RuntimeContext<'a, 'catalog> {
    pub bootstrap: BootstrapContext<'a, 'catalog>,
    pub store: &'a BinaryStore,
    pub account: AccountId,
    pub lease_ms: i64,
}

pub(crate) async fn run(
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    config: SweepConfig,
    credentials: &ProfileCredentials,
) -> Result<SweepSummary, SweepError> {
    config.validate()?;
    store.validate_catalog(catalog.raw_bytes()).await?;
    let sweep = sweep_id(store, &config).await?;
    let client = Client::new(transport_config(&config)?)?;
    let settings = executor_config(config.concurrency)?;
    let lease_ms = lease_budget(settings)?;
    let executor = ApiExecutor::new(settings)?;
    let home = RegionId::parse(credentials.region())
        .map_err(|_| SweepError::InvalidConfig("profile region is invalid"))?;
    let bootstrap = BootstrapContext {
        catalog,
        client: &client,
        executor: &executor,
        credentials,
        home: &home,
        config: &config,
    };
    let collection = collect(&bootstrap, store, &sweep, lease_ms);
    tokio::pin!(collection);
    tokio::select! {
        signal = tokio::signal::ctrl_c() => {
            executor.cancel();
            if signal.is_err() { return Err(SweepError::Operation("Ctrl-C cancellation handler failed")); }
            Err(SweepError::Api(inventory_engine::limiter::ApiError::Cancelled))
        }
        result = &mut collection => result,
    }
}

async fn collect(
    bootstrap: &BootstrapContext<'_, '_>,
    store: &BinaryStore,
    sweep: &SweepId,
    lease_ms: i64,
) -> Result<SweepSummary, SweepError> {
    let account = bootstrap_account(bootstrap).await?;
    let context = RuntimeContext {
        bootstrap: BootstrapContext {
            catalog: bootstrap.catalog,
            client: bootstrap.client,
            executor: bootstrap.executor,
            credentials: bootstrap.credentials,
            home: bootstrap.home,
            config: bootstrap.config,
        },
        store,
        account,
        lease_ms,
    };
    let regions = discover_regions(&context).await?;
    seed_enumerations(&context, sweep, &regions).await?;
    drain_work(&context, sweep).await
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
    let attempts = u64::from(settings.max_attempts);
    let attempt_time = settings
        .attempt_timeout_ms
        .checked_mul(attempts)
        .ok_or(SweepError::Operation("executor attempt budget overflow"))?;
    let retry_count = attempts.saturating_sub(1);
    let backoff = settings
        .max_backoff_ms
        .checked_mul(retry_count)
        .ok_or(SweepError::Operation("executor backoff budget overflow"))?;
    let rate = u64::from(settings.requests_per_second);
    let concurrent = u64::try_from(settings.global_concurrency)
        .map_err(|_| SweepError::Operation("executor concurrency exceeds lease budget range"))?;
    let admissions = concurrent
        .checked_add(rate.saturating_sub(1))
        .and_then(|value| value.checked_div(rate))
        .ok_or(SweepError::Operation("executor admission budget overflow"))?;
    let pacing = admissions
        .checked_mul(1_000)
        .and_then(|value| value.checked_mul(attempts))
        .ok_or(SweepError::Operation("executor pacing budget overflow"))?;
    let total = attempt_time
        .checked_add(backoff)
        .and_then(|value| value.checked_add(pacing))
        .and_then(|value| value.checked_add(LEASE_MARGIN_MS))
        .ok_or(SweepError::Operation("executor lease budget overflow"))?;
    i64::try_from(total)
        .map_err(|_| SweepError::Operation("executor lease budget exceeds storage range"))
}

async fn sweep_id(store: &BinaryStore, config: &SweepConfig) -> Result<SweepId, SweepError> {
    if config.resume {
        return store
            .resume_sweep()
            .await?
            .ok_or(SweepError::Operation("no pending sweep to resume"));
    }
    SweepId::new(uuid::Uuid::new_v4().to_string()).map_err(SweepError::from)
}

async fn bootstrap_account(context: &BootstrapContext<'_, '_>) -> Result<AccountId, SweepError> {
    let plan = unique_purpose(context.catalog, |purpose| {
        matches!(purpose, PlanPurpose::AccountIdentity { .. })
    })?;
    let PlanPurpose::AccountIdentity { account_path } = plan.purpose() else {
        return Err(SweepError::Operation(
            "account identity plan purpose changed",
        ));
    };
    let endpoint = plan.endpoint(context.home)?;
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

async fn discover_regions(context: &RuntimeContext<'_, '_>) -> Result<Vec<Region>, SweepError> {
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
    let endpoint = plan.endpoint(context.bootstrap.home)?;
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
