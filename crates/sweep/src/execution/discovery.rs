mod lease;
pub(crate) use lease::lease;
use super::{InventoryStore, RuntimeContext};
use crate::{CredentialSource, SweepError, call, input, record};
use inventory_core::{
    catalog::runtime::{CallablePlan, RequestRouting, ResolvedEndpoint},
    record::{Record, ValueRef},
};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

const MAX_LEASES: usize = 256;
const MAX_ENDPOINTS: usize = 16;
const MAX_TTL_MINUTES: i64 = 1440;

#[derive(Default)]
pub(crate) struct EndpointCache(Mutex<BTreeMap<(String, String, String), Lease>>);
struct Lease {
    endpoint: String,
    expires: Instant,
}

impl EndpointCache {
    async fn get(&self, key: &(String, String, String), now: Instant) -> Option<String> {
        self.0
            .lock()
            .await
            .get(key)
            .filter(|lease| lease.expires > now)
            .map(|lease| lease.endpoint.clone())
    }
    async fn insert(
        &self,
        key: (String, String, String),
        endpoint: String,
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

pub(super) async fn resolve(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    plan: &CallablePlan<'_>,
    mut endpoint: ResolvedEndpoint,
) -> Result<ResolvedEndpoint, SweepError> {
    let RequestRouting::Discovered { operation, .. } = plan.routing() else {
        return Ok(endpoint);
    };
    let key = cache_key(context, plan, &endpoint);
    let now = Instant::now();
    if let Some(url) = context.endpoints.get(&key, now).await {
        endpoint.url = url;
        return Ok(endpoint);
    }
    let response = discover(context, plan, operation, &endpoint).await?;
    let (url, ttl) = endpoint_lease(plan, &response.record, &endpoint.signing_region)?;
    context.endpoints.insert(key, url.clone(), ttl, now).await?;
    endpoint.url = url;
    Ok(endpoint)
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
    let discovery = context.bootstrap.catalog.lookup(plan.service_name(), operation).ok_or(
        SweepError::Operation("reviewed endpoint discovery operation is missing"),
    )?;
    let region = inventory_core::inventory::RegionId::parse(&endpoint.signing_region)?;
    let endpoint = discovery.endpoint(
        &region,
        &inventory_core::inventory::Region::Regional(region.clone()),
    )?;
    discovery_call(context, discovery, &endpoint, region).await
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
        url: &url,
        base: &base,
        fields: &[],
        metrics: bootstrap.metrics,
    }
    .run()
    .await
}

fn endpoint_lease(
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
