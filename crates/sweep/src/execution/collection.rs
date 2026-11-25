use super::*;
use crate::ProgressEvent;
use crate::control::Progress;
use crate::metrics::{Accumulators, complete, elapsed_ms};
use std::time::Instant;

pub(super) struct Resources {
    client: Client,
    pub executor: ApiExecutor,
    home: RegionId,
    lease_ms: i64,
    metrics: Accumulators,
    resolved_endpoints: super::resolved_endpoints::ResolvedEndpoints,
}

impl Resources {
    pub(super) fn new(
        config: &SweepConfig,
        credentials: &impl CredentialSource,
    ) -> Result<Self, SweepError> {
        config.validate()?;
        let client = Client::new(transport_config(config, descriptors::soft_open_files())?)?;
        let settings = executor_config(config.concurrency)?;
        let lease_ms = lease_budget(settings)?;
        let executor = ApiExecutor::with_default_policy(
            settings,
            rate::policy(
                config.max_rps,
                config.max_rps_milli,
                settings.per_bucket_concurrency,
            ),
            std::collections::BTreeMap::new(),
        )?;
        let home = RegionId::parse(credentials.region())
            .map_err(|_| SweepError::InvalidConfig("profile region is invalid"))?;
        Ok(Self {
            client,
            executor,
            home,
            lease_ms,
            metrics: Accumulators::default(),
            resolved_endpoints: Default::default(),
        })
    }

    pub(super) fn set_attempts(
        &mut self,
        attempts: Option<std::sync::Arc<crate::AttemptedOperations>>,
    ) {
        self.metrics.attempts = attempts;
    }

    fn complete(&self, metrics: &mut crate::SweepMetrics) {
        complete(
            metrics,
            &self.metrics,
            self.executor.stats(),
            self.client.statistics(),
        );
    }

    fn bootstrap<'a, 'catalog, C: CredentialSource>(
        &'a self,
        catalog: &'a RuntimeCatalog<'catalog>,
        config: &'a SweepConfig,
        credentials: &'a C,
    ) -> BootstrapContext<'a, 'catalog, C> {
        BootstrapContext {
            catalog,
            client: &self.client,
            executor: &self.executor,
            credentials,
            home: &self.home,
            config,
            metrics: &self.metrics,
            resolved_endpoints: &self.resolved_endpoints,
        }
    }
}

pub(super) async fn collect(
    catalog: &RuntimeCatalog<'_>,
    store: &impl InventoryStore,
    config: &SweepConfig,
    credentials: &impl CredentialSource,
    resources: &Resources,
    progress: &Progress,
) -> Result<SweepSummary, SweepError> {
    let bootstrap = resources.bootstrap(catalog, config, credentials);
    store.validate_catalog(catalog.raw_bytes()).await?;
    let started = Instant::now();
    let (sweep, account) = initialize(&bootstrap, store).await?;
    let mut metrics = crate::SweepMetrics {
        bootstrap_ms: elapsed_ms(started),
        ..Default::default()
    };
    let context = context(bootstrap, store, account, resources.lease_ms, progress)?;
    let mut summary = drain(&context, &sweep, &mut metrics).await?;
    resources.complete(&mut metrics);
    summary.metrics = metrics;
    Ok(summary)
}

async fn initialize(
    bootstrap: &BootstrapContext<'_, '_, impl CredentialSource>,
    store: &impl InventoryStore,
) -> Result<(SweepId, AccountId), SweepError> {
    let account = bootstrap_account(bootstrap).await?;
    let sweep = sweep_id(store, bootstrap.config, &account).await?;
    store.create_sweep_manifest(&sweep, &account).await?;
    Ok((sweep, account))
}

fn context<'a, 'catalog, C: CredentialSource, S: InventoryStore>(
    bootstrap: BootstrapContext<'a, 'catalog, C>,
    store: &'a S,
    account: AccountId,
    lease_ms: i64,
    progress: &'a Progress,
) -> Result<RuntimeContext<'a, 'catalog, C, S>, SweepError> {
    let consumers =
        page::consumer_index::ConsumerIndex::build(bootstrap.catalog).map_err(|error| {
            tracing::error!(%error, "reviewed consumer index construction failed");
            SweepError::Operation("reviewed consumer index construction failed")
        })?;
    Ok(RuntimeContext {
        bootstrap,
        store,
        account,
        lease_ms,
        endpoints: discovery::EndpointCache::default(),
        consumers,
        progress,
    })
}

async fn drain(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    metrics: &mut crate::SweepMetrics,
) -> Result<SweepSummary, SweepError> {
    let started = Instant::now();
    let regions = discover_regions(context).await?;
    let root_scopes = seed_enumerations(context, sweep, &regions).await?;
    let region_count = u32::try_from(regions.len())
        .map_err(|_| SweepError::Operation("region count exceeds progress range"))?;
    context.progress.emit(|| ProgressEvent::Started {
        sweep: sweep.as_str().to_owned(),
        account: context.account.as_str().to_owned(),
        regions: region_count,
        root_scopes,
    });
    metrics.seed_ms = elapsed_ms(started);
    let budget = match context.bootstrap.config.slice_timeout_secs {
        Some(seconds) => SliceBudget::starting_at(now_ms()?, seconds),
        None => SliceBudget::unbounded(),
    };
    timed_drain(context, sweep, budget, metrics).await
}

async fn timed_drain(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    budget: SliceBudget,
    metrics: &mut crate::SweepMetrics,
) -> Result<SweepSummary, SweepError> {
    let started = Instant::now();
    let summary = drain_work(context, sweep, budget).await;
    metrics.drain_ms = elapsed_ms(started);
    summary
}
