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
    let RequestRouting::Discovered {
        operation,
        endpoints_path,
        address_path,
        cache_minutes_path,
        trusted_dns_suffix,
    } = plan.routing()
    else {
        return Ok(endpoint);
    };
    let key = (
        plan.service_name().to_owned(),
        context.account.as_str().to_owned(),
        endpoint.signing_region.clone(),
    );
    let now = Instant::now();
    if let Some(url) = context.endpoints.get(&key, now).await {
        endpoint.url = url;
        return Ok(endpoint);
    }
    let discovery = context
        .bootstrap
        .catalog
        .lookup(plan.service_name(), operation)
        .ok_or(SweepError::Operation(
            "reviewed endpoint discovery operation is missing",
        ))?;
    let region = inventory_core::inventory::RegionId::parse(&endpoint.signing_region)?;
    let bootstrap_endpoint = discovery.endpoint(
        &region,
        &inventory_core::inventory::Region::Regional(region.clone()),
    )?;
    let url = super::endpoint_url(context.bootstrap.config, &bootstrap_endpoint.url)?;
    let base = input::root_bindings(
        discovery,
        Some(&context.account),
        Some(&inventory_core::inventory::Region::Regional(region)),
    )?;
    let response = call::Call {
        client: context.bootstrap.client,
        executor: context.bootstrap.executor,
        credentials: context.bootstrap.credentials,
        plan: discovery,
        endpoint: &bootstrap_endpoint,
        url: &url,
        base: &base,
        fields: &[],
    }
    .run()
    .await?;
    let suffix = trusted_dns_suffix.replace("{region}", &endpoint.signing_region);
    let (url, ttl) = lease(
        &response.record,
        endpoints_path,
        address_path,
        cache_minutes_path,
        &suffix,
    )?;
    context.endpoints.insert(key, url.clone(), ttl, now).await?;
    endpoint.url = url;
    Ok(endpoint)
}

pub(crate) fn lease(
    response: &Record,
    endpoints: &[inventory_core::record::FieldId],
    address: &[inventory_core::record::FieldId],
    minutes: &[inventory_core::record::FieldId],
    suffix: &str,
) -> Result<(String, Duration), SweepError> {
    let rows = record::item_records(
        response,
        &inventory_core::catalog::runtime::ItemPlan::Path(endpoints.to_vec().into_boxed_slice()),
    )?;
    if rows.is_empty() || rows.len() > MAX_ENDPOINTS {
        return Err(SweepError::Operation(
            "endpoint discovery returned invalid endpoint count",
        ));
    }
    let mut selected = None;
    for row in &rows {
        let Some(ValueRef::Text(address)) = record::path_value(row.root(), address)? else {
            return Err(SweepError::Operation(
                "discovered endpoint address is missing",
            ));
        };
        let Some(ValueRef::Int(minutes)) = record::path_value(row.root(), minutes)? else {
            return Err(SweepError::Operation("discovered endpoint TTL is missing"));
        };
        if !(1..=MAX_TTL_MINUTES).contains(&minutes) {
            return Err(SweepError::Operation(
                "discovered endpoint TTL exceeds bounds",
            ));
        }
        let seconds = u64::try_from(minutes)
            .map_err(|_| SweepError::Operation("discovered endpoint TTL is negative"))?
            .checked_mul(60)
            .ok_or(SweepError::Operation("discovered endpoint TTL overflow"))?;
        if selected.is_none() {
            selected = Some((trusted_url(address, suffix)?, Duration::from_secs(seconds)));
        } else {
            trusted_host(address, suffix)?;
        }
    }
    selected.ok_or(SweepError::Operation(
        "endpoint discovery returned no endpoints",
    ))
}

fn trusted_url(address: &str, suffix: &str) -> Result<String, SweepError> {
    trusted_host(address, suffix)?;
    Ok(format!("https://{address}"))
}

fn trusted_host(address: &str, suffix: &str) -> Result<(), SweepError> {
    if address.len() > 253
        || !suffix.starts_with('.')
        || !address.ends_with(suffix)
        || address.len() <= suffix.len()
        || !address.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
    {
        return Err(SweepError::Operation(
            "discovered endpoint violates trusted HTTPS host policy",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn service_discovery_never_accepts_url_authority_injection() {
        let suffix = ".timestream.us-east-1.amazonaws.com";
        assert!(trusted_url("ingest-cell1.timestream.us-east-1.amazonaws.com", suffix).is_ok());
        for address in [
            "http://ingest-cell1.timestream.us-east-1.amazonaws.com",
            "ingest-cell1.timestream.us-east-1.amazonaws.com.evil.test",
            "127.0.0.1",
            "user@ingest-cell1.timestream.us-east-1.amazonaws.com",
            "ingest-cell1.timestream.us-east-1.amazonaws.com:443",
            "ingest-cell1.timestream.us-west-2.amazonaws.com",
        ] {
            assert!(trusted_url(address, suffix).is_err(), "{address}");
        }
    }
    #[tokio::test]
    async fn leases_expire_and_are_service_account_region_isolated()
    -> Result<(), Box<dyn std::error::Error>> {
        let cache = EndpointCache::default();
        let now = Instant::now();
        let key = (
            "timestream-write".to_owned(),
            "123456789012".to_owned(),
            "us-east-1".to_owned(),
        );
        cache
            .insert(
                key.clone(),
                "https://ingest-cell1.timestream.us-east-1.amazonaws.com".to_owned(),
                Duration::from_secs(60),
                now,
            )
            .await?;
        assert!(cache.get(&key, now).await.is_some());
        assert!(
            cache
                .get(
                    &key,
                    now.checked_add(Duration::from_secs(60))
                        .ok_or("clock overflow")?
                )
                .await
                .is_none()
        );
        let query = ("timestream-query".to_owned(), key.1.clone(), key.2.clone());
        assert!(cache.get(&query, now).await.is_none());
        let account = (key.0.clone(), "999999999999".to_owned(), key.2.clone());
        assert!(cache.get(&account, now).await.is_none());
        let region = (key.0.clone(), key.1.clone(), "us-west-2".to_owned());
        assert!(cache.get(&region, now).await.is_none());
        Ok(())
    }

    #[test]
    fn endpoint_leases_require_bounded_count_ttl_and_every_trusted_address()
    -> Result<(), Box<dyn std::error::Error>> {
        use inventory_core::record::{FieldId, Key, Value};
        let endpoints = FieldId::new(1)?;
        let address = FieldId::new(2)?;
        let minutes = FieldId::new(3)?;
        let suffix = ".timestream.us-east-1.amazonaws.com";
        let item = |host: &str, ttl| {
            Value::Map(BTreeMap::from([
                (Key::Field(address), Value::Text(host.to_owned())),
                (Key::Field(minutes), Value::Int(ttl)),
            ]))
        };
        let response = |items| {
            Record::encode(&Value::Map(BTreeMap::from([(
                Key::Field(endpoints),
                Value::List(items),
            )])))
        };
        let valid = "ingest-cell1.timestream.us-east-1.amazonaws.com";
        let record = response(vec![item(valid, 60)])?;
        assert_eq!(
            lease(&record, &[endpoints], &[address], &[minutes], suffix)?,
            (format!("https://{valid}"), Duration::from_secs(3600))
        );
        for ttl in [-1, 0, 1441, i64::MAX] {
            let record = response(vec![item(valid, ttl)])?;
            assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
        }
        for count in [0, 17] {
            let record = response((0..count).map(|_| item(valid, 60)).collect())?;
            assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
        }
        let record = response(vec![item(valid, 60), item("evil.example", 60)])?;
        assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
        Ok(())
    }
}
