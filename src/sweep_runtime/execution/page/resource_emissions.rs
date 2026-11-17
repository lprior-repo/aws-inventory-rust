use super::{PageParts, PageRequest, WorkLease, assembly, rows};
use crate::sweep_runtime::{SweepError, record, resource_projection};
use futures_util::{stream, StreamExt, TryStreamExt};
use inventory_core::{catalog::runtime::{CallablePlan, ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan, ResourceEmissionPlan}, inventory::{DiscoveryContext, DiscoveryFrame}, record::Record};

/// Every descriptor consumes this same decoded response; this module cannot make a request.
pub(super) async fn additional(
    context: &assembly::Context<'_, '_>, lease: &WorkLease, plan: &CallablePlan<'_>,
    response: &Record, request: &PageRequest, ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    if plan.resource_emissions().len() == 1 { return Ok(PageParts::default()); }
    let projected = if plan.resource_emissions().iter().skip(1).any(|emission| needs_current(emission.identity(), plan)) {
        let frame = DiscoveryFrame::project(lease.scope().service().clone(),
            lease.scope().operation().clone(), 0, plan.context_projection(), response, &request.record, &request.record)
            .map_err(|_| SweepError::Operation("reviewed emitter response projection failed"))?;
        Some(ancestry.push(frame).map_err(|_| SweepError::Operation("emitter ancestry exceeds bounds"))?)
    } else { None };
    let ancestry = projected.as_ref().unwrap_or(ancestry);
    stream::iter(plan.resource_emissions().iter().skip(1)).map(Ok::<_, SweepError>)
        .try_fold((PageParts::default(), 0), |(mut parts, mut tags), emission| {
            async move {
                let next = descriptor(context, lease, plan, emission, response, request, ancestry).await?;
                assembly::append(&mut parts, next, &mut tags)?;
                Ok((parts, tags))
            }
        }).await.map(|(parts, _)| parts)
}

async fn descriptor(context: &assembly::Context<'_, '_>, lease: &WorkLease,
    plan: &CallablePlan<'_>, emission: &ResourceEmissionPlan, response: &Record,
    request: &PageRequest, ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    let (count, selected) = record::item_selection(response, emission.items())?;
    if count > assembly::MAX_PAGE_ROWS { return Err(SweepError::Operation("page exceeds the atomic observation bound")); }
    stream::iter(0..count.div_ceil(assembly::ROWS_PER_CHECKPOINT)).map(Ok::<_, SweepError>)
        .try_fold((PageParts::default(), 0, selected), |(mut parts, mut tags, mut selected), _| async move {
            assembly::checkpoint(context.executor).await?;
            let raw = selected.by_ref().take(assembly::ROWS_PER_CHECKPOINT).collect::<Result<Vec<_>, _>>()?;
            let next = chunk(context, lease, plan, emission, raw, request, ancestry)?;
            assembly::append(&mut parts, next, &mut tags)?;
            Ok((parts, tags, selected))
        }).await.map(|(parts, _, _)| parts)
}

pub(super) fn chunk(context: &assembly::Context<'_, '_>, lease: &WorkLease,
    plan: &CallablePlan<'_>, emission: &ResourceEmissionPlan, raw: Vec<Record>,
    request: &PageRequest, ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    let items = selected(emission, raw)?;
    let observations = items.iter().map(|row| rows::emission_observation(rows::EmissionScope {
        catalog: context.catalog,
        identity_context: record::IdentityContext { account: context.account, region: &request.region },
        plan, emission, region: lease.scope().region(), request: &request.record, ancestry,
    }, row))
        .collect::<Result<Vec<_>, _>>()?;
    let references = rows::references(context.catalog, plan, &request.region, &items, &observations, &request.record, lease.scope().parent())?;
    let tags = rows::emission_tags(emission, &items, &observations)?;
    Ok(PageParts { observations, references, tags, ..PageParts::default() })
}

pub(in crate::sweep_runtime) fn selected(emission: &ResourceEmissionPlan, raw: Vec<Record>)
    -> Result<Vec<Record>, SweepError> {
    raw.into_iter().map(|row| match emission.projection() {
        Some(projection) => resource_projection::project_with(projection, &row),
        None => Ok(row),
    }).collect()
}

fn needs_current(identity: &IdentityPlan, plan: &CallablePlan<'_>) -> bool {
    let fields = |fields: &[IdentityFieldPlan]| fields.iter().any(|field|
        matches!(field, IdentityFieldPlan::Context { service, operation, source, .. }
            if service.as_ref() == plan.service_name() && operation.as_ref() == plan.operation_name()
                && *source != ContextSource::SelectedItem));
    let leaf = |leaf: &IdentityLeafPlan| matches!(leaf, IdentityLeafPlan::Fields(values) if fields(values));
    match identity {
        IdentityPlan::CorrelatedKey(values) => fields(values),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => leaf(fallback),
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) =>
            branches.iter().any(|branch| leaf(&branch.identity)),
        _ => false,
    }
}
