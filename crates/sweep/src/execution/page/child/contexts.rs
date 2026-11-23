use super::*;

#[cfg(test)]
pub(in crate::execution::page) fn contexts(
    plan: &CallablePlan<'_>,
    rows: &[Record],
    request: &Record,
    response: &Record,
    ancestry: &DiscoveryContext,
    scope: &ScopeKey,
) -> Result<Vec<DiscoveryContext>, SweepError> {
    let mut budget = PageContextBudget::new(ancestry, rows.len())?;
    contexts_from(plan, Rows { items: rows, offset: 0 }, request, response, ancestry, scope, &mut budget)
}

#[cfg(test)]
pub(in crate::execution::page) fn contexts_from(
    plan: &CallablePlan<'_>,
    rows: Rows<'_>,
    request: &Record,
    response: &Record,
    ancestry: &DiscoveryContext,
    scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<DiscoveryContext>, SweepError> {
    let end = rows.offset.checked_add(rows.items.len()).ok_or(SweepError::Operation(
        "discovery item ordinal exceeds bound",
    ))?;
    contexts_at(plan, (rows.offset..end).zip(rows.items.iter()), request, response, ancestry, scope, budget)
}

pub(in crate::execution::page) fn contexts_at<'row>(
    plan: &CallablePlan<'_>,
    rows: impl ExactSizeIterator<Item = (usize, &'row Record)>,
    request: &Record,
    response: &Record,
    ancestry: &DiscoveryContext,
    scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<DiscoveryContext>, SweepError> {
    let mut contexts = Vec::new();
    contexts.try_reserve(rows.len())?;
    for (ordinal, row) in rows {
        let frame = project_frame(plan, ordinal, row, request, response, scope)?;
        let bytes = ancestry.pushed_encoded_len(&frame)
            .map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?;
        budget.charge(bytes)?;
        contexts.push(ancestry.push(frame)
            .map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?);
    }
    Ok(contexts)
}

fn project_frame(
    plan: &CallablePlan<'_>,
    ordinal: usize,
    row: &Record,
    request: &Record,
    response: &Record,
    scope: &ScopeKey,
) -> Result<DiscoveryFrame, SweepError> {
    let ordinal = u32::try_from(ordinal)
        .map_err(|_| SweepError::Operation("discovery item ordinal exceeds bound"))?;
    DiscoveryFrame::project(
        scope.service().clone(), scope.operation().clone(), ordinal,
        plan.context_projection(), row, response, request,
    )
    .and_then(|frame| frame.with_branch(scope.branch()))
    .map_err(|_| SweepError::Operation("reviewed discovery projection failed"))
}
