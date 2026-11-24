use super::{InventoryStore, RuntimeContext, cursor};
use crate::{CredentialSource, SweepError, input};
use inventory_core::{catalog::runtime::CallablePlan, record::Record};
use inventory_engine::inventory::WorkLease;

pub(super) struct Prepared {
    pub(super) endpoint: super::super::discovery::RequestEndpoint,
    pub(super) base: Record,
    pub(super) fields: Vec<input::InputField>,
    pub(super) region: inventory_core::inventory::RegionId,
    scope: inventory_core::inventory::Region,
}

pub(super) async fn prepare(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    cursor: &cursor::Cursor,
) -> Result<Prepared, SweepError> {
    let (endpoint, region) = endpoint(context, lease, plan).await?;
    let base = bindings(lease.bindings())?;
    let fields = request_fields(cursor, plan, context.bootstrap.config.max_page_size)?;
    Ok(Prepared {
        endpoint,
        base,
        fields,
        region,
        scope: lease.scope().region().clone(),
    })
}

async fn endpoint(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
) -> Result<
    (
        super::super::discovery::RequestEndpoint,
        inventory_core::inventory::RegionId,
    ),
    SweepError,
> {
    let region = super::super::routing::request_region(
        plan,
        lease.scope().region(),
        context.bootstrap.home,
        lease.scope().branch(),
    )?;
    let endpoint = context
        .bootstrap
        .resolved_endpoints
        .resolve(context.bootstrap.catalog, plan, &region)
        .await?;
    let endpoint = super::super::discovery::resolve(context, plan, endpoint).await?;
    Ok((endpoint, region))
}

fn bindings(bytes: &[u8]) -> Result<Record, SweepError> {
    if bytes.is_empty() {
        Record::encode(&inventory_core::record::Value::Map(
            std::collections::BTreeMap::new(),
        ))
        .map_err(SweepError::from)
    } else {
        Record::from_bytes(bytes.to_vec()).map_err(SweepError::from)
    }
}

fn request_fields(
    cursor: &cursor::Cursor,
    plan: &CallablePlan<'_>,
    max_page_size: Option<u32>,
) -> Result<Vec<input::InputField>, SweepError> {
    let mut fields = cursor.request_fields(plan.pagination())?;
    if let Some(size) = plan.pagination().page_size.as_ref() {
        fields.push(input::InputField {
            path: size.path.clone(),
            value: size.capped(max_page_size),
        });
    }
    Ok(fields)
}

impl Prepared {
    pub(super) async fn send(
        &self,
        context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
        plan: &CallablePlan<'_>,
    ) -> Result<crate::call::DecodedResponse, SweepError> {
        crate::call::Call {
            client: context.bootstrap.client,
            executor: context.bootstrap.executor,
            credentials: context.bootstrap.credentials,
            plan,
            endpoint: &self.endpoint.modeled,
            url: super::super::endpoint_url(context.bootstrap.config, self.endpoint.url())?,
            base: &self.base,
            fields: &self.fields,
            metrics: context.bootstrap.metrics,
            account: context.account.as_str(),
            scope: &self.scope,
        }
        .run()
        .await
    }
}

pub(super) fn ancestry(
    bytes: &[u8],
    expected: Option<inventory_core::record::ContentHash>,
) -> Result<inventory_core::inventory::DiscoveryContext, SweepError> {
    let ancestry = inventory_core::inventory::DiscoveryContext::decode(bytes)
        .map_err(|_| SweepError::Operation("persisted discovery context is invalid"))?;
    let actual = (!bytes.is_empty())
        .then(|| ancestry.discriminator())
        .transpose()
        .map_err(|_| SweepError::Operation("persisted discovery context digest failed"))?;
    if expected != actual {
        return Err(SweepError::Operation(
            "persisted discovery context digest mismatch",
        ));
    }
    Ok(ancestry)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_context_is_checked_before_request_preparation()
    -> Result<(), Box<dyn std::error::Error>> {
        let context = inventory_core::inventory::DiscoveryContext::empty();
        let bytes = context.encode()?;
        assert_eq!(ancestry(&bytes, Some(context.discriminator()?))?, context);
        assert!(ancestry(&bytes, None).is_err());
        assert!(ancestry(&[255], None).is_err());
        assert!(ancestry(&[], Some(context.discriminator()?)).is_err());
        assert_eq!(
            ancestry(&[], None)?,
            inventory_core::inventory::DiscoveryContext::empty()
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "preparation/wire_tests.rs"]
mod wire_tests;
