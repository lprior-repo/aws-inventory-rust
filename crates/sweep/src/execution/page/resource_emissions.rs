mod descriptor;
use descriptor::Descriptor;

use super::{PageParts, PageRequest, WorkLease, assembly, rows};
use crate::{SweepError, record, resource_projection};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan,
        IdentityPlan, ResourceEmissionPlan,
    },
    inventory::{DiscoveryContext, DiscoveryFrame},
    record::Record,
};

pub(super) async fn additional(
    context: &assembly::Context<'_, '_>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    response: &Record,
    request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    if plan.resource_emissions().len() == 1 {
        return Ok(PageParts::default());
    }
    let projected = projected_ancestry(lease, plan, response, request, ancestry)?;
    let ancestry = projected.as_ref().unwrap_or(ancestry);
    collect_additional(context, lease, plan, response, request, ancestry).await
}

async fn collect_additional(
    context: &assembly::Context<'_, '_>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    response: &Record,
    request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    stream::iter(plan.resource_emissions().iter().skip(1))
        .map(Ok::<_, SweepError>)
        .try_fold(
            (PageParts::default(), 0),
            |(mut parts, mut tags), emission| async move {
                let next =
                    descriptor(context, lease, plan, emission, response, request, ancestry).await?;
                assembly::append(&mut parts, next, &mut tags)?;
                Ok((parts, tags))
            },
        )
        .await
        .map(|(parts, _)| parts)
}

fn projected_ancestry(
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    response: &Record,
    request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<Option<DiscoveryContext>, SweepError> {
    if !plan
        .resource_emissions()
        .iter()
        .skip(1)
        .any(|emission| needs_current(emission.identity(), plan))
    {
        return Ok(None);
    }
    let frame = emitter_frame(lease, plan, response, request)?;
    ancestry
        .push(frame)
        .map(Some)
        .map_err(|_| SweepError::Operation("emitter ancestry exceeds bounds"))
}

fn emitter_frame(
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    response: &Record,
    request: &PageRequest,
) -> Result<DiscoveryFrame, SweepError> {
    DiscoveryFrame::project(
        lease.scope().service().clone(),
        lease.scope().operation().clone(),
        0,
        plan.context_projection(),
        response,
        &request.record,
        &request.record,
    )
    .map_err(|_| SweepError::Operation("reviewed emitter response projection failed"))
}

async fn descriptor(
    context: &assembly::Context<'_, '_>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    emission: &ResourceEmissionPlan,
    response: &Record,
    request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    Descriptor::new(context, lease, plan, emission, request, ancestry)
        .collect(response)
        .await
}

pub(crate) fn selected(
    emission: &ResourceEmissionPlan,
    raw: Vec<Record>,
) -> Result<Vec<Record>, SweepError> {
    raw.into_iter()
        .map(|row| match emission.projection() {
            Some(projection) => resource_projection::project_with(projection, &row),
            None => Ok(row),
        })
        .collect()
}

fn needs_current(identity: &IdentityPlan, plan: &CallablePlan<'_>) -> bool {
    let fields = |fields: &[IdentityFieldPlan]| {
        fields.iter().any(|field|
        matches!(field, IdentityFieldPlan::Context { service, operation, source, .. }
            if service.as_ref() == plan.service_name() && operation.as_ref() == plan.operation_name()
                && *source != ContextSource::SelectedItem))
    };
    let leaf = |leaf: &IdentityLeafPlan| matches!(leaf, IdentityLeafPlan::Fields(values) if fields(values));
    match identity {
        IdentityPlan::CorrelatedKey(values) => fields(values),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => leaf(fallback),
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            branches.iter().any(|branch| leaf(&branch.identity))
        }
        _ => false,
    }
}
