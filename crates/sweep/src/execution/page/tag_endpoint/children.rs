use super::*;
use inventory_core::catalog::runtime::{TagEndpointParentPlan, TagEndpointRequestPlan};

pub(in crate::execution::page) struct ChildContext<'a, 'catalog> {
    pub parent_plan: &'a CallablePlan<'catalog>,
    pub plan: &'a CallablePlan<'catalog>,
    pub request: &'a Record,
    pub sweep: &'a SweepId,
    pub discovery: &'a [u8],
    pub branch: Option<&'a ExecutionBranch>,
}

pub(in crate::execution::page) fn child_work(
    context: &ChildContext<'_, '_>,
    row: &Record,
    observation: &impl AsRef<ResourceKey>,
    budget: &mut super::super::child::PageContextBudget,
) -> Result<Option<WorkSpec>, SweepError> {
    let Some(source) = child_source(context) else {
        return Ok(None);
    };
    let observation = observation.as_ref();
    validate_kind(context.parent_plan, row, observation)?;
    let fields = source
        .request
        .as_ref()
        .map(|request| request_field(context, row, observation, request))
        .transpose()?;
    if matches!(fields, Some(None)) {
        budget.skip_request();
        return Ok(None);
    }
    let fields = fields.flatten().into_iter().collect::<Vec<_>>();
    budget.charge(context.discovery.len())?;
    make_child(context, observation, fields).map(Some)
}

fn request_field(
    context: &ChildContext<'_, '_>,
    row: &Record,
    observation: &ResourceKey,
    request: &TagEndpointRequestPlan,
) -> Result<Option<input::InputField>, SweepError> {
    request_value(context, row, observation, request).map(|value| {
        value.map(|value| input::InputField {
            path: request.input.clone(),
            value: Value::Text(value.to_owned()),
        })
    })
}

fn child_source<'a>(context: &'a ChildContext<'_, '_>) -> Option<&'a TagEndpointParentPlan> {
    let PlanPurpose::TagEndpoint(endpoint) = context.plan.purpose() else {
        return None;
    };
    if context.plan.service_name() != context.parent_plan.service_name()
        || !matches!(context.parent_plan.purpose(), PlanPurpose::Enumeration)
    {
        return None;
    }
    endpoint
        .parents
        .iter()
        .find(|source| source.operation.as_ref() == context.parent_plan.operation_name())
}

fn validate_kind(
    plan: &CallablePlan<'_>,
    row: &Record,
    observation: &ResourceKey,
) -> Result<(), SweepError> {
    let kind = plan.classification().map_or_else(
        || Ok(plan.kind()),
        |classification| classification.classify(row),
    )?;
    if kind != &observation.kind {
        return Err(SweepError::Operation(
            "tag endpoint parent kind differs from observation",
        ));
    }
    Ok(())
}

fn request_value<'a>(
    context: &'a ChildContext<'_, '_>,
    row: &'a Record,
    observation: &'a ResourceKey,
    request: &TagEndpointRequestPlan,
) -> Result<Option<&'a str>, SweepError> {
    let value = match request.source {
        TagEndpointSource::Response => record::path_value(row.root(), &request.parent)?,
        TagEndpointSource::Request => record::path_value(context.request.root(), &request.parent)?,
        TagEndpointSource::OwnerIdentity => return Ok(Some(observation.id.as_str())),
    };
    match value {
        Some(ValueRef::Text("")) => Ok(None),
        Some(ValueRef::Text(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(SweepError::Operation("tag endpoint source is not a string")),
    }
}

fn make_child(
    context: &ChildContext<'_, '_>,
    observation: &ResourceKey,
    fields: Vec<input::InputField>,
) -> Result<WorkSpec, SweepError> {
    let parent = observation;
    let base = input::root_bindings_branch(
        context.plan,
        context.branch,
        Some(&parent.account),
        Some(&parent.region),
    )?;
    let scope = child_scope(context, parent)?;
    let bindings = input::merge_input(&base, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(), scope, None, bindings)?
        .with_context(context.discovery.to_vec())
        .map_err(SweepError::from)
}

fn child_scope(
    context: &ChildContext<'_, '_>,
    parent: &ResourceKey,
) -> Result<ScopeKey, SweepError> {
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
    discovery_scope(scope, context.discovery)
}

fn discovery_scope(scope: ScopeKey, bytes: &[u8]) -> Result<ScopeKey, SweepError> {
    if bytes.is_empty() {
        return Ok(scope);
    }
    let discovery = DiscoveryContext::decode(bytes)
        .map_err(|_| SweepError::Operation("tag work discovery context is invalid"))?;
    Ok(scope.with_context(
        discovery.discriminator().map_err(|_| {
            SweepError::Operation("tag work discovery context cannot be identified")
        })?,
    ))
}
