use super::{RuntimeContext, cursor};
use crate::{SweepError, input};
use inventory_core::{
    catalog::runtime::{CallablePlan, ResolvedEndpoint},
    record::Record,
};
use inventory_engine::inventory::WorkLease;

pub(super) async fn prepare(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    cursor: &cursor::Cursor,
) -> Result<
    (
        ResolvedEndpoint,
        String,
        Record,
        Vec<input::InputField>,
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
    let endpoint = super::super::routing::resolve(plan, &region)?;
    let endpoint = super::super::discovery::resolve(context, plan, endpoint).await?;
    let url = context
        .bootstrap
        .config
        .endpoint_override()?
        .unwrap_or(&endpoint.url)
        .to_owned();
    let base = if lease.bindings().is_empty() {
        Record::encode(&inventory_core::record::Value::Map(
            std::collections::BTreeMap::new(),
        ))?
    } else {
        Record::from_bytes(lease.bindings().to_vec())?
    };
    let mut fields = cursor.request_fields(plan.pagination())?;
    if let Some(size) = plan.pagination().page_size.as_ref() {
        fields.push(input::InputField {
            path: size.path.clone(),
            value: size.value(),
        });
    }
    Ok((endpoint, url, base, fields, region))
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
