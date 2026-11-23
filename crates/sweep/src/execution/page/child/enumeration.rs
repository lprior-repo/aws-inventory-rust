use super::*;

pub(super) fn child_for_row(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
    discovery: &[u8],
    budget: &mut PageContextBudget,
) -> Result<Option<WorkSpec>, SweepError> {
    if !eligible(context, child, row, branch)? {
        return Ok(None);
    }
    let scope_kind = branch.map_or(child.scope(), |branch| branch.scope);
    let scope_region = child_region(scope_kind, row.scope.region(), context.home);
    let request_region = super::super::super::routing::request_region(
        child,
        &scope_region,
        context.home,
        branch.map(|branch| branch.discriminator.as_str()),
    )?;
    let Some(fields) = fields(context, child, row, branch, &request_region)? else {
        return Ok(None);
    };
    budget.charge(discovery.len())?;
    make_child(context, child, row, branch, fields, discovery).map(Some)
}

fn eligible(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
) -> Result<bool, SweepError> {
    if !child.effective_bindings(branch).any(|binding| {
        binding::consumes_current(&binding.source, context.parent_plan, row.scope.branch())
    }) {
        return Ok(false);
    }
    crate::row_policy::child(
        child,
        branch,
        context.parent_plan,
        row.row,
        row.ancestry,
        row.scope,
    )
}

fn fields(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
    request_region: &RegionId,
) -> Result<Option<Vec<input::InputField>>, SweepError> {
    let mut fields = Vec::new();
    for binding in child.effective_bindings(branch) {
        let (_, value) = resolve_binding(context, request_region, row, &binding.source)?;
        let Some(value) = value else {
            return Ok(None);
        };
        fields.try_reserve(1)?;
        fields.push(input::InputField {
            path: binding.path.clone(),
            value,
        });
    }
    Ok(Some(fields))
}

pub(super) fn child_region(scope: PlanScope, parent: &Region, home: &RegionId) -> Region {
    match scope {
        PlanScope::Account => Region::Global,
        PlanScope::HomeRegion => Region::Regional(home.clone()),
        PlanScope::Region => match parent {
            Region::Global => Region::Regional(home.clone()),
            Region::Regional(region) => Region::Regional(region.clone()),
        },
    }
}

fn make_child(
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
    fields: Vec<input::InputField>,
    discovery: &[u8],
) -> Result<WorkSpec, SweepError> {
    let region = child_region(
        branch.map_or(plan.scope(), |branch| branch.scope),
        row.scope.region(),
        context.home,
    );
    let scope = child_scope(context, plan, row, branch, region)?;
    let bindings = input::merge_input(&empty_record()?, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(), scope, None, bindings)?
        .with_context(discovery.to_vec())
        .map_err(SweepError::from)
}

fn child_scope(
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
    region: Region,
) -> Result<ScopeKey, SweepError> {
    let mut scope = ScopeKey::new(
        context.account.clone(),
        region,
        ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,
        row.parent.or(row.scope.parent()).cloned(),
    )?;
    if let Some(branch) = branch.map(|branch| branch.discriminator.as_str()) {
        scope = scope.with_branch(branch.to_owned())?;
    }
    Ok(scope.with_context(
        row.ancestry
            .discriminator()
            .map_err(|_| SweepError::Operation("discovery context digest failed"))?,
    ))
}

pub(super) fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}
