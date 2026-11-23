use super::*;

pub(super) struct Chunk {
    pub(super) ordinals: [usize; ROWS_PER_CHECKPOINT],
    pub(super) items: Vec<inventory_core::record::Record>,
    pub(super) raw_count: usize,
}

impl Chunk {
    fn new() -> Self {
        Self {
            ordinals: [0; ROWS_PER_CHECKPOINT],
            items: Vec::new(),
            raw_count: 0,
        }
    }
}

pub(super) fn select(
    page: &page::Page<'_, '_>,
    selected: &mut impl Iterator<Item = Result<inventory_core::record::Record, SweepError>>,
    offset: usize,
) -> Result<Chunk, SweepError> {
    let mut chunk = Chunk::new();
    let mut retained = chunk.ordinals.iter_mut();
    for (ordinal, row) in selected.by_ref().take(ROWS_PER_CHECKPOINT).enumerate() {
        chunk.raw_count = add_ordinal(chunk.raw_count, 1)?;
        let row = row?;
        if !selected_row(page, &row)? {
            continue;
        }
        let ordinal = add_ordinal(offset, ordinal)?;
        *retained.next().ok_or(SweepError::Operation(
            "discovery item ordinal exceeds bound",
        ))? = ordinal;
        chunk.items.try_reserve(1)?;
        chunk
            .items
            .push(resource_projection::project(page.plan, &row)?);
    }
    Ok(chunk)
}

fn selected_row(
    page: &page::Page<'_, '_>,
    row: &inventory_core::record::Record,
) -> Result<bool, SweepError> {
    crate::row_policy::selected(
        page.plan,
        page.branch,
        row,
        &page.request.record,
        &page.response.record,
    )
}

pub(super) fn add_ordinal(offset: usize, count: usize) -> Result<usize, SweepError> {
    offset.checked_add(count).ok_or(SweepError::Operation(
        "discovery item ordinal exceeds bound",
    ))
}
