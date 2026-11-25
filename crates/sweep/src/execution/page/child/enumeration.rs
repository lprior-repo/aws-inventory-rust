use super::*;

pub(super) fn child_for_row(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>,
    discovery: &[u8],
    budget: &mut PageContextBudget,
) -> Result<Option<WorkSpec>, SweepError> {
    let binding_row = row.bindings();
    let bindings = context.bindings();
    if !eligible(bindings.parent_plan, child, &binding_row, branch)? {
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
    let resolution = fields(&bindings, child, &binding_row, branch, &request_region)?;
    let fields = match resolution {
        FieldResolution::Absent => return Ok(None),
        FieldResolution::Empty => {
            budget.skip_request();
            return Ok(None);
        }
        FieldResolution::Ready(fields) => fields,
    };
    budget.charge(discovery.len())?;
    make_child(context, child, row, branch, fields, discovery).map(Some)
}

pub(crate) fn eligible(
    parent: &CallablePlan<'_>,
    child: &CallablePlan<'_>,
    row: &BindingRow<'_>,
    branch: Option<&ExecutionBranch>,
) -> Result<bool, SweepError> {
    if !child
        .effective_bindings(branch)
        .any(|binding| consumes_current(&binding.source, parent, row.branch))
    {
        return Ok(false);
    }
    crate::row_policy::child(child, branch, parent, row.row, row.ancestry, row.branch)
}

pub(crate) enum FieldResolution {
    Ready(Vec<input::InputField>),
    Absent,
    Empty,
}

pub(crate) fn fields(
    context: &BindingContext<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BindingRow<'_>,
    branch: Option<&ExecutionBranch>,
    request_region: &RegionId,
) -> Result<FieldResolution, SweepError> {
    let mut fields = Vec::new();
    for binding in child.effective_bindings(branch) {
        let (_, value) = binding::resolve(context, request_region, row, &binding.source)?;
        match value {
            None => return Ok(FieldResolution::Absent),
            Some(Value::Text(text)) if text.is_empty() => return Ok(FieldResolution::Empty),
            Some(value) => {
                fields.try_reserve(1)?;
                fields.push(input::InputField {
                    path: binding.path.clone(),
                    value,
                });
            }
        }
    }
    Ok(FieldResolution::Ready(fields))
}

pub(crate) fn child_region(scope: PlanScope, parent: &Region, home: &RegionId) -> Region {
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
    let scope = scope_for_row(context, plan, row, branch, region)?;
    let bindings = input::merge_input(&empty_record()?, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(), scope, None, bindings)?
        .with_context(discovery.to_vec())
        .map_err(SweepError::from)
}

fn scope_for_row(
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

pub(crate) fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}
