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
    selected
        .by_ref()
        .take(ROWS_PER_CHECKPOINT)
        .enumerate()
        .try_for_each(|(ordinal, row)| {
            chunk.raw_count = add_ordinal(chunk.raw_count, 1)?;
            let Some(row) = resource_projection::select_item(
                page.plan,
                page.branch,
                row?,
                &page.request.record,
                &page.response.record,
            )?
            else {
                return Ok::<_, SweepError>(());
            };
            let ordinal = add_ordinal(offset, ordinal)?;
            *retained.next().ok_or(SweepError::Operation(
                "discovery item ordinal exceeds bound",
            ))? = ordinal;
            chunk.items.try_reserve(1)?;
            chunk.items.push(row);
            Ok(())
        })?;
    Ok(chunk)
}

pub(super) fn add_ordinal(offset: usize, count: usize) -> Result<usize, SweepError> {
    offset.checked_add(count).ok_or(SweepError::Operation(
        "discovery item ordinal exceeds bound",
    ))
}

#[cfg(test)]
mod tests;
