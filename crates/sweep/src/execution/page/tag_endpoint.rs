mod accumulation;
mod children;
pub(super) use children::{ChildContext, child_work};
#[cfg(test)]
mod tests;

use super::{PageParts, cursor};
use crate::{SweepError, call, input, record};
use accumulation::Accumulation;
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ExecutionBranch, PlanPurpose, RuntimeCatalog, TagEndpointSource,
    },
    inventory::{AccountId, DiscoveryContext, Kind, OperationId, ResourceKey, ScopeKey, ServiceId},
    record::{Record, Value, ValueRef},
};
use inventory_engine::inventory::{SweepId, WorkLease, WorkSpec};

#[cfg(test)]
use inventory_core::inventory::Observation;

pub(super) fn decode_cursor(token: Option<&[u8]>) -> Result<cursor::Cursor, SweepError> {
    cursor::Cursor::decode(token)
}

pub(super) fn prepare_page(
    catalog: &RuntimeCatalog<'_>,
    account: &AccountId,
    max_pages: u32,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    cursor: cursor::Cursor,
    response: call::DecodedResponse,
) -> Result<PageParts, SweepError> {
    let endpoint = endpoint(plan)?;
    validate_completion(plan, &response.record)?;
    let owner = owner(catalog, account, lease, plan)?;
    let page = record::tag_observation(&response.record, owner, &endpoint.tags)?;
    let pending = Accumulation::restore(owner, lease.token())?.append(page)?;
    let next = cursor.next(
        plan.pagination(),
        &response.record,
        &response.headers,
        max_pages,
    )?;
    finish(pending, next)
}

fn finish(pending: Accumulation, next: Option<cursor::Advance>) -> Result<PageParts, SweepError> {
    let tags = vec![pending.observation(next.is_none())?];
    let next = next
        .map(|advance| {
            Ok::<_, SweepError>(cursor::Advance {
                cursor: pending.checkpoint(advance.cursor)?,
                continuation_digest: advance.continuation_digest,
            })
        })
        .transpose()?;
    Ok(PageParts {
        observations: Vec::new(),
        references: Vec::new(),
        tags,
        children: Vec::new(),
        next,
        rows: 0,
        skipped_child_requests: 0,
    })
}

fn endpoint<'a>(
    plan: &'a CallablePlan<'_>,
) -> Result<&'a inventory_core::catalog::runtime::TagEndpointPlan, SweepError> {
    match plan.purpose() {
        PlanPurpose::TagEndpoint(endpoint) => Ok(endpoint),
        _ => Err(SweepError::Operation(
            "tag endpoint has no reviewed purpose",
        )),
    }
}

fn validate_completion(plan: &CallablePlan<'_>, response: &Record) -> Result<(), SweepError> {
    if let Some(path) = &plan.pagination().truncated_path
        && !matches!(
            record::path_value(response.root(), path)?,
            Some(ValueRef::Bool(_))
        )
    {
        return Err(SweepError::Operation(
            "tag response omitted explicit pagination completion evidence",
        ));
    }
    Ok(())
}

fn owner<'a>(
    catalog: &RuntimeCatalog<'_>,
    account: &AccountId,
    lease: &'a WorkLease,
    plan: &CallablePlan<'_>,
) -> Result<&'a ResourceKey, SweepError> {
    let parent = lease.scope().parent().ok_or(SweepError::Operation(
        "tag endpoint requires trusted parent work",
    ))?;
    validate_owner_scope(account, lease, parent)?;
    validate_owner_kind(catalog, plan, parent)?;
    Ok(parent)
}

fn validate_owner_scope(
    account: &AccountId,
    lease: &WorkLease,
    parent: &ResourceKey,
) -> Result<(), SweepError> {
    if &parent.account != account
        || lease.scope().account() != &parent.account
        || lease.scope().region() != &parent.region
    {
        return Err(SweepError::Operation(
            "tag endpoint owner scope differs from trusted parent",
        ));
    }
    Ok(())
}

fn validate_owner_kind(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    parent: &ResourceKey,
) -> Result<(), SweepError> {
    let endpoint = endpoint(plan)?;
    let matched = endpoint.parents.iter().any(|source| {
        catalog
            .lookup(plan.service_name(), &source.operation)
            .is_some_and(|source| {
                matches!(source.purpose(), PlanPurpose::Enumeration)
                    && accepts_kind(source, &parent.kind)
            })
    });
    if !matched {
        return Err(SweepError::Operation(
            "tag endpoint owner kind has no reviewed parent source",
        ));
    }
    Ok(())
}

fn accepts_kind(plan: &CallablePlan<'_>, kind: &Kind) -> bool {
    plan.kind() == kind
        || plan.classification().is_some_and(|classification| {
            classification
                .variants
                .iter()
                .any(|(_, candidate)| candidate == kind)
        })
}
