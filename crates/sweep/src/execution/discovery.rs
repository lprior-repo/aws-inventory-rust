mod lease;
use super::{InventoryStore, RuntimeContext};
use crate::{CredentialSource, SweepError, call, input, record};
use inventory_core::{
    catalog::runtime::{CallablePlan, RequestRouting, ResolvedEndpoint, RuntimeCatalog},
    record::{Record, ValueRef},
};
pub(crate) use lease::lease;
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

const MAX_LEASES: usize = 256;
const MAX_ENDPOINTS: usize = 16;
const MAX_TTL_MINUTES: i64 = 1440;

#[derive(Default)]
pub(crate) struct EndpointCache(Mutex<BTreeMap<(String, String, String), Lease>>);
struct Lease {
    endpoint: Arc<str>,
    expires: Instant,
}

impl EndpointCache {
    async fn get(&self, key: &(String, String, String), now: Instant) -> Option<Arc<str>> {
        self.0
            .lock()
            .await
            .get(key)
            .filter(|lease| lease.expires > now)
            .map(|lease| Arc::clone(&lease.endpoint))
    }
    async fn insert(
        &self,
        key: (String, String, String),
        endpoint: Arc<str>,
        ttl: Duration,
        now: Instant,
    ) -> Result<(), SweepError> {
        let expires = now
            .checked_add(ttl)
            .ok_or(SweepError::Operation("endpoint lease expiry overflow"))?;
        let mut cache = self.0.lock().await;
        cache.retain(|_, lease| lease.expires > now);
        if cache.len() >= MAX_LEASES && !cache.contains_key(&key) {
            let oldest = cache
                .iter()
                .min_by_key(|(_, lease)| lease.expires)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                cache.remove(&oldest);
            }
        }
        cache.insert(key, Lease { endpoint, expires });
        Ok(())
    }
}

pub(super) struct RequestEndpoint {
    pub(super) modeled: Arc<ResolvedEndpoint>,
    discovered_url: Option<Arc<str>>,
}

impl RequestEndpoint {
    pub(super) fn url(&self) -> &str {
        self.discovered_url.as_deref().unwrap_or(&self.modeled.url)
    }
}

pub(super) async fn resolve(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    plan: &CallablePlan<'_>,
    endpoint: Arc<ResolvedEndpoint>,
) -> Result<RequestEndpoint, SweepError> {
    let RequestRouting::Discovered { operation, .. } = plan.routing() else {
        return Ok(RequestEndpoint {
            modeled: endpoint,
            discovered_url: None,
        });
    };
    let key = cache_key(context, plan, &endpoint);
    let now = Instant::now();
    if let Some(url) = context.endpoints.get(&key, now).await {
        return Ok(RequestEndpoint {
            modeled: endpoint,
            discovered_url: Some(url),
        });
    }
    let response = discover(context, plan, operation, &endpoint).await?;
    let (url, ttl) = endpoint_lease(plan, &response.record, &endpoint.signing_region)?;
    let url = Arc::<str>::from(url);
    context
        .endpoints
        .insert(key, Arc::clone(&url), ttl, now)
        .await?;
    Ok(RequestEndpoint {
        modeled: endpoint,
        discovered_url: Some(url),
    })
}

fn cache_key(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    plan: &CallablePlan<'_>,
    endpoint: &ResolvedEndpoint,
) -> (String, String, String) {
    (
        plan.service_name().to_owned(),
        context.account.as_str().to_owned(),
        endpoint.signing_region.clone(),
    )
}

async fn discover(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    plan: &CallablePlan<'_>,
    operation: &str,
    endpoint: &ResolvedEndpoint,
) -> Result<call::DecodedResponse, SweepError> {
    let discovery = lookup(context.bootstrap.catalog, plan, operation)?;
    let region = inventory_core::inventory::RegionId::parse(&endpoint.signing_region)?;
    let endpoint = resolve_endpoint(
        context.bootstrap.catalog,
        discovery,
        &region,
        context.bootstrap.resolved_endpoints,
    )
    .await?;
    discovery_call(context, discovery, &endpoint, region).await
}

pub(crate) fn lookup<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
    plan: &CallablePlan<'_>,
    operation: &str,
) -> Result<&'a CallablePlan<'catalog>, SweepError> {
    catalog
        .lookup(plan.service_name(), operation)
        .ok_or(SweepError::Operation(
            "reviewed endpoint discovery operation is missing",
        ))
}

pub(super) async fn resolve_endpoint(
    catalog: &inventory_core::catalog::runtime::RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    region: &inventory_core::inventory::RegionId,
    endpoints: &super::resolved_endpoints::ResolvedEndpoints,
) -> Result<Arc<ResolvedEndpoint>, SweepError> {
    endpoints.resolve(catalog, plan, region).await
}

async fn discovery_call(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    plan: &CallablePlan<'_>,
    endpoint: &ResolvedEndpoint,
    region: inventory_core::inventory::RegionId,
) -> Result<call::DecodedResponse, SweepError> {
    let bootstrap = &context.bootstrap;
    let url = super::endpoint_url(bootstrap.config, &endpoint.url)?;
    let region = inventory_core::inventory::Region::Regional(region);
    let base = input::root_bindings(plan, Some(&context.account), Some(&region))?;
    call::Call {
        client: bootstrap.client,
        executor: bootstrap.executor,
        credentials: bootstrap.credentials,
        plan,
        endpoint,
        url,
        emulator_endpoint_override: bootstrap.config.endpoint_override()?.is_some(),
        base: &base,
        fields: &[],
        metrics: bootstrap.metrics,
        account: context.account.as_str(),
        scope: &region,
    }
    .run()
    .await
}

pub(crate) fn endpoint_lease(
    plan: &CallablePlan<'_>,
    response: &Record,
    region: &str,
) -> Result<(String, Duration), SweepError> {
    match plan.routing() {
        RequestRouting::Discovered {
            endpoints_path,
            address_path,
            cache_minutes_path,
            trusted_dns_suffix,
            ..
        } => lease(
            response,
            endpoints_path,
            address_path,
            cache_minutes_path,
            &trusted_dns_suffix.replace("{region}", region),
        ),
        _ => Err(SweepError::Operation("endpoint discovery routing changed")),
    }
}

#[cfg(test)]
mod tests;
