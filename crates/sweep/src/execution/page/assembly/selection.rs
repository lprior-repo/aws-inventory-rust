use super::*;
use inventory_core::catalog::runtime::ExecutionBranch;

pub(super) struct Chunk {
    pub(super) ordinals: [usize; ROWS_PER_CHECKPOINT],
    pub(super) items: Vec<inventory_core::record::Record>,
    pub(super) raw_count: usize,
}

pub(super) fn select(
    page: &page::Page<'_, '_>,
    selected: &mut impl Iterator<Item = Result<inventory_core::record::Record, SweepError>>,
    offset: usize,
) -> Result<Chunk, SweepError> {
    let mut chunk = Chunk { ordinals: [0; ROWS_PER_CHECKPOINT], items: Vec::new(), raw_count: 0 };
    let mut retained = chunk.ordinals.iter_mut();
    for (ordinal, row) in selected.by_ref().take(ROWS_PER_CHECKPOINT).enumerate() {
        chunk.raw_count = add_ordinal(chunk.raw_count, 1)?;
        let row = row?;
        if !selected_row(page.plan, page.branch, &row, page.request, &page.response.record)? {
            continue;
        }
        let ordinal = add_ordinal(offset, ordinal)?;
        *retained.next().ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))? = ordinal;
        chunk.items.try_reserve(1)?;
        chunk.items.push(resource_projection::project(page.plan, &row)?);
    }
    Ok(chunk)
}

fn selected_row(
    plan: &CallablePlan<'_>,
    branch: Option<&ExecutionBranch>,
    row: &inventory_core::record::Record,
    request: &PageRequest,
    response: &inventory_core::record::Record,
) -> Result<bool, SweepError> {
    crate::row_policy::selected(plan, branch, row, &request.record, response)
}

pub(super) fn add_ordinal(offset: usize, count: usize) -> Result<usize, SweepError> {
    offset.checked_add(count).ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))
}
