mod accumulation;
#[cfg(test)]
mod tests;

use super::{PageParts, cursor};
use crate::{SweepError, call, input, record};
use accumulation::Accumulation;
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ExecutionBranch, PlanPurpose, RuntimeCatalog, TagEndpointSource,
    },
    inventory::{
        AccountId, DiscoveryContext, Kind, Observation, OperationId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::{Record, Value, ValueRef},
};
use inventory_engine::inventory::{SweepId, WorkLease, WorkSpec};

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
    let PlanPurpose::TagEndpoint(endpoint) = plan.purpose() else {
        return Err(SweepError::Operation(
            "tag endpoint has no reviewed purpose",
        ));
    };
    if let Some(path) = &plan.pagination().truncated_path
        && !matches!(
            record::path_value(response.record.root(), path)?,
            Some(ValueRef::Bool(_))
        )
    {
        return Err(SweepError::Operation(
            "tag response omitted explicit pagination completion evidence",
        ));
    }
    let owner = owner(catalog, account, lease, plan)?;
    let page = record::tag_observation(&response.record, owner, &endpoint.tags)?;
    let pending = Accumulation::restore(owner, lease.token())?.append(page)?;
    let next = cursor.next(
        plan.pagination(),
        &response.record,
        &response.headers,
        max_pages,
    )?;
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
    })
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
    if &parent.account != account
        || lease.scope().account() != &parent.account
        || lease.scope().region() != &parent.region
    {
        return Err(SweepError::Operation(
            "tag endpoint owner scope differs from trusted parent",
        ));
    }
    let PlanPurpose::TagEndpoint(endpoint) = plan.purpose() else {
        return Err(SweepError::Operation(
            "tag endpoint has no reviewed purpose",
        ));
    };
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
    Ok(parent)
}

pub(super) struct ChildContext<'a, 'catalog> {
    pub parent_plan: &'a CallablePlan<'catalog>,
    pub plan: &'a CallablePlan<'catalog>,
    pub request: &'a Record,
    pub sweep: &'a SweepId,
    pub discovery: &'a [u8],
    pub branch: Option<&'a ExecutionBranch>,
}

pub(super) fn child_work(
    context: &ChildContext<'_, '_>,
    row: &Record,
    observation: &Observation,
    budget: &mut super::child::PageContextBudget,
) -> Result<Option<WorkSpec>, SweepError> {
    let PlanPurpose::TagEndpoint(endpoint) = context.plan.purpose() else {
        return Ok(None);
    };
    if context.plan.service_name() != context.parent_plan.service_name()
        || !matches!(context.parent_plan.purpose(), PlanPurpose::Enumeration)
    {
        return Ok(None);
    }
    let Some(source) = endpoint
        .parents
        .iter()
        .find(|source| source.operation.as_ref() == context.parent_plan.operation_name())
    else {
        return Ok(None);
    };
    let kind = context.parent_plan.classification().map_or_else(
        || Ok(context.parent_plan.kind()),
        |classification| classification.classify(row),
    )?;
    if kind != &observation.key().kind {
        return Err(SweepError::Operation(
            "tag endpoint parent kind differs from observation",
        ));
    }
    let fields = source
        .request
        .as_ref()
        .map(|request| {
            let value = match request.source {
                TagEndpointSource::Response => record::path_value(row.root(), &request.parent)?,
                TagEndpointSource::Request => {
                    record::path_value(context.request.root(), &request.parent)?
                }
                TagEndpointSource::OwnerIdentity => {
                    return Ok(Some(input::InputField {
                        path: request.input.clone(),
                        value: Value::Text(observation.key().id.as_str().to_owned()),
                    }));
                }
            };
            match value {
                Some(ValueRef::Text(value)) => Ok(Some(input::InputField {
                    path: request.input.clone(),
                    value: Value::Text(value.to_owned()),
                })),
                None => Ok(None),
                _ => Err(SweepError::Operation("tag endpoint source is not a string")),
            }
        })
        .transpose()?;
    if matches!(fields, Some(None)) {
        return Ok(None);
    }
    let fields = fields.flatten().into_iter().collect::<Vec<_>>();
    budget.charge(context.discovery.len())?;
    make_child(context, observation, fields).map(Some)
}

fn make_child(
    context: &ChildContext<'_, '_>,
    observation: &Observation,
    fields: Vec<input::InputField>,
) -> Result<WorkSpec, SweepError> {
    let parent = observation.key();
    let base = input::root_bindings_branch(
        context.plan,
        context.branch,
        Some(&parent.account),
        Some(&parent.region),
    )?;
    let scope = ScopeKey::new(
        parent.account.clone(),
        parent.region.clone(),
        ServiceId::parse(context.plan.service_name())?,
        OperationId::parse(context.plan.operation_name())?,
        Some(parent.clone()),
    )?;
    let scope = match context.branch {
        Some(branch) => scope.with_branch(branch.discriminator.clone())?,
        None => scope,
    };
    let scope = if context.discovery.is_empty() {
        scope
    } else {
        let discovery = DiscoveryContext::decode(context.discovery)
            .map_err(|_| SweepError::Operation("tag work discovery context is invalid"))?;
        scope.with_context(discovery.discriminator().map_err(|_| {
            SweepError::Operation("tag work discovery context cannot be identified")
        })?)
    };
    let bindings = input::merge_input(&base, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(), scope, None, bindings)?
        .with_context(context.discovery.to_vec())
        .map_err(SweepError::from)
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
