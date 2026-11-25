use super::*;

pub(in crate::execution::page) struct WorkBatch<'a, T: AsRef<ResourceKey>> {
    pub observations: &'a [T],
    pub request: &'a Record,
    pub contexts: &'a [DiscoveryContext],
    pub scope: &'a ScopeKey,
}

struct BoundRow<'a> {
    row: RowContext<'a>,
    observation: Option<&'a ResourceKey>,
    request: &'a Record,
    discovery: &'a [u8],
}

pub(in crate::execution::page) fn work(
    context: &Context<'_, '_>,
    rows: &[Record],
    observations: &[impl AsRef<ResourceKey>],
    request: &Record,
    contexts: &[DiscoveryContext],
    scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<WorkSpec>, SweepError> {
    work_with_row_children(
        context,
        rows,
        WorkBatch {
            observations,
            request,
            contexts,
            scope,
        },
        budget,
        |_, _, _| Ok(()),
    )
}

pub(in crate::execution::page) fn work_with_row_children(
    context: &Context<'_, '_>,
    rows: &[Record],
    batch: WorkBatch<'_, impl AsRef<ResourceKey>>,
    budget: &mut PageContextBudget,
    mut observe: impl FnMut(&Record, &DiscoveryContext, &mut [WorkSpec]) -> Result<(), SweepError>,
) -> Result<Vec<WorkSpec>, SweepError> {
    let mut children = Vec::new();
    rows.iter().enumerate().try_for_each(|(ordinal, row)| {
        let start = children.len();
        let ancestry = batch
            .contexts
            .get(ordinal)
            .ok_or(SweepError::Operation("discovery item context is absent"))?;
        append_row(
            context,
            row,
            ancestry,
            ordinal,
            &batch,
            budget,
            &mut children,
        )?;
        let added = children
            .get_mut(start..)
            .ok_or(SweepError::Operation("child work range is absent"))?;
        observe(row, ancestry, added)
    })?;
    Ok(children)
}

fn append_row(
    context: &Context<'_, '_>,
    row: &Record,
    next: &DiscoveryContext,
    ordinal: usize,
    batch: &WorkBatch<'_, impl AsRef<ResourceKey>>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    let discovery = next
        .encode()
        .map_err(|_| SweepError::Operation("discovery ancestry encoding failed"))?;
    let observation = batch.observations.get(ordinal).map(AsRef::as_ref);
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
    observation: Option<&'a ResourceKey>,
    scope: &'a ScopeKey,
    ancestry: &'a DiscoveryContext,
) -> RowContext<'a> {
    RowContext {
        row,
        parent: observation,
        scope,
        ancestry,
    }
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
    for branch in child
        .branches()
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| child.branches().is_empty()))
    {
        if let Some(work) =
            enumeration::child_for_row(context, child, &row.row, branch, row.discovery, budget)?
        {
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
    let Some(observation) = row.observation else {
        return Ok(());
    };
    let target_stage = target_stage(child, row.row.scope.branch());
    for branch in child
        .branches()
        .iter()
        .filter(|branch| target_stage.is_none_or(|source| branch.discriminator.as_str() == source))
        .map(Some)
        .chain(child.branches().is_empty().then_some(None))
    {
        append_tag_branch(context, child, row, observation, branch, budget, children)?;
    }
    Ok(())
}

fn target_stage<'a>(child: &CallablePlan<'_>, source: Option<&'a str>) -> Option<&'a str> {
    source.filter(|source| {
        child
            .branches()
            .iter()
            .any(|target| target.discriminator.as_str() == *source)
    })
}

fn append_tag_branch(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    observation: &ResourceKey,
    branch: Option<&ExecutionBranch>,
    budget: &mut PageContextBudget,
    children: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    if !selected_tag_row(context, child, row, branch)? {
        return Ok(());
    }
    let tag_context = tag_context(context, child, row, branch);
    if let Some(work) =
        super::super::tag_endpoint::child_work(&tag_context, row.row.row, observation, budget)?
    {
        push(children, work)?;
    }
    Ok(())
}

fn selected_tag_row(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &BoundRow<'_>,
    branch: Option<&ExecutionBranch>,
) -> Result<bool, SweepError> {
    crate::row_policy::child(
        child,
        branch,
        context.parent_plan,
        row.row.row,
        row.row.ancestry,
        row.row.scope,
    )
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
        return Err(SweepError::Operation(
            "page exceeds the atomic child-work bound",
        ));
    }
    children.try_reserve(1)?;
    children.push(work);
    Ok(())
}
