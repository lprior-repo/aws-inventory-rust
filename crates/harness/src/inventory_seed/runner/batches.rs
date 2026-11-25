use super::*;
use std::ops::Range;

pub(super) fn ranges(
    prepared: &[PreparedStep<'_>],
    steps: &[super::super::SeedStep],
    concurrency: usize,
) -> Result<Vec<Range<usize>>, SeedError> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for _ in 0..steps.len() {
        if start == steps.len() {
            break;
        }
        let first = steps
            .get(start)
            .ok_or(SeedError::Invalid("seed batch start is missing"))?;
        let length = length(prepared, steps, first, start, concurrency)?;
        let end = start
            .checked_add(length)
            .ok_or(SeedError::Invalid("seed batch overflow"))?;
        ranges.push(start..end);
        start = end;
    }
    Ok(ranges)
}

fn length(
    prepared: &[PreparedStep<'_>],
    steps: &[super::super::SeedStep],
    first: &super::super::SeedStep,
    start: usize,
    concurrency: usize,
) -> Result<usize, SeedError> {
    let tail = steps
        .get(start..)
        .ok_or(SeedError::Invalid("seed batch tail is missing"))?;
    let mut length = 0_usize;
    for step in tail.iter().take(concurrency) {
        if !independent(step, first, prepared, start)? {
            break;
        }
        length = length
            .checked_add(1)
            .ok_or(SeedError::Invalid("seed batch length overflow"))?;
    }
    if length == 0 {
        return Err(SeedError::Invalid("seed batch cannot make progress"));
    }
    Ok(length)
}

fn independent(
    step: &super::super::SeedStep,
    first: &super::super::SeedStep,
    prepared: &[PreparedStep<'_>],
    start: usize,
) -> Result<bool, SeedError> {
    if step.service != first.service || step.operation != first.operation {
        return Ok(false);
    }
    let model = prepared
        .get(usize::from(step.id.0))
        .ok_or(SeedError::Invalid("seed prepared batch step is missing"))?;
    Ok(!super::dependencies::dependencies(model)?
        .iter()
        .any(|id| usize::from(id.0) >= start))
}
