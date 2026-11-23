use super::*;

struct Batch<'a> {
    observations: &'a [Observation],
    request: &'a Record,
    contexts: &'a [DiscoveryContext],
    scope: &'a ScopeKey,
}

struct BoundRow<'a> {
    row: RowContext<'a>,
    observation: Option<&'a Observation>,
    request: &'a Record,
    discovery: &'a [u8],
}

pub(in crate::execution::page) fn work(
    context: &Context<'_, '_>,
    rows: &[Record],
    observations: &[Observation],
    request: &Record,
    contexts: &[DiscoveryContext],
    scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<WorkSpec>, SweepError> {
    let mut children = Vec::new();
    let batch = Batch { observations, request, contexts, scope };
    for (ordinal, row) in rows.iter().enumerate() {
        append_row(context, row, ordinal, &batch, budget, &mut children)?;
    }
    Ok(children)
}

fn append_row(
    context: &Context<'_, '_>,
    row: &Record,
    ordinal: usize,
    batch: &Batch<'_>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    let next = batch.contexts.get(ordinal)
        .ok_or(SweepError::Operation("discovery item context is absent"))?;
    let discovery = next.encode()
        .map_err(|_| SweepError::Operation("discovery ancestry encoding failed"))?;
    let observation = batch.observations.get(ordinal);
    let row = BoundRow {
        row: row_context(row, observation, batch.scope, next),
        observation,
        request: batch.request,
        discovery: &discovery,
    };
    append_candidates(context, &row, budget, children)
}

fn append_candidates(
    context: &Context<'_, '_>,
    row: &BoundRow<'_>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    for child in context.consumers.candidates_for_branch(
        context.parent_plan.service_name(),
        context.parent_plan.operation_name(),
        row.row.scope.branch(),
    ) {
        append_candidate(context, child, row, budget, children)?;
    }
    Ok(())
}

fn row_context<'a>(
    row: &'a Record,
    observation: Option<&'a Observation>,
    scope: &'a ScopeKey,
    ancestry: &'a DiscoveryContext,
) -> RowContext<'a> {
    RowContext { row, parent: observation.map(Observation::key), scope, ancestry }
}

fn append_candidate(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    match child.purpose() {
        PlanPurpose::Enumeration => append_enumeration(context, child, row, budget, children),
        PlanPurpose::TagEndpoint(_) => append_tags(context, child, row, budget, children),
        _ => Ok(()),
    }
}

fn append_enumeration(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    for branch in child.branches().iter().map(Some)
        .chain(std::iter::once(None).filter(|_| child.branches().is_empty())) {
        if let Some(work) = enumeration::child_for_row(
            context, child, &row.row, branch, row.discovery, budget,
        )? {
            push(children, work)?;
        }
    }
    Ok(())
}

fn append_tags(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    let Some(observation) = row.observation else { return Ok(()); };
    let target_stage = row.row.scope.branch().filter(|source| {
        child.branches().iter().any(|target| target.discriminator.as_str() == *source)
    });
    for branch in child.branches().iter()
        .filter(|branch| target_stage.is_none_or(|source| branch.discriminator.as_str() == source))
        .map(Some).chain(child.branches().is_empty().then_some(None)) {
        append_tag_branch(context, child, row, observation, branch, budget, children)?;
    }
    Ok(())
}

fn append_tag_branch(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    observation: &Observation,
    branch: Option<&ExecutionBranch>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    if !crate::row_policy::child(
        child, branch, context.parent_plan, row.row.row, row.row.ancestry, row.row.scope,
    )? { return Ok(()); }
    let tag_context = tag_context(context, child, row, branch);
    if let Some(work) = super::super::tag_endpoint::child_work(
        &tag_context, row.row.row, observation, budget,
    )? { push(children, work)?; }
    Ok(())
}

fn tag_context<'a, 'catalog>(
    context: &'a Context<'_, 'catalog>,
    child: &'a CallablePlan<'catalog>,
    row: &'a BoundRow<'_>,
    branch: Option<&'a ExecutionBranch>,
) -> super::super::tag_endpoint::ChildContext<'a, 'catalog> {
    super::super::tag_endpoint::ChildContext {
        parent_plan: context.parent_plan,
        plan: child,
        request: row.request,
        sweep: context.sweep,
        discovery: row.discovery,
        branch,
    }
}

fn push(children: &mut Vec<WorkSpec>, work: WorkSpec) -> Result<(), SweepError> {
    if children.len() >= MAX_PAGE_CHILDREN {
        return Err(SweepError::Operation("page exceeds the atomic child-work bound"));
    }
    children.try_reserve(1)?;
    children.push(work);
    Ok(())
}
