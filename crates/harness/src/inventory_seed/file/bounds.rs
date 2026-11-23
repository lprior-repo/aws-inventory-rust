use super::*;

const MAX_EXPANDED_BYTES: usize = 67_108_864;

pub(super) fn validate(document: &Document<'_>) -> Result<(), SeedError> {
    let (steps, nodes, bytes) = document.steps.iter().try_fold(
        (0_usize, 0_usize, 0_usize),
        |(steps, nodes, bytes), step| {
            let (count, size) = inputs(step, document.service)?;
            Ok::<_, SeedError>((
                add(steps, 1, step.repeat)?,
                add(nodes, count, step.repeat)?,
                add(bytes, size, step.repeat)?,
            ))
        },
    )?;
    let (relationships, relation_bytes) = relationships(document)?;
    let bytes = add(bytes, relation_bytes, 1)?;
    if steps > 4096 || nodes > 65_536 || relationships > 4096 || bytes > MAX_EXPANDED_BYTES {
        return Err(SeedError::Invalid(
            "seed expansion exceeds step, relationship, input or byte limits",
        ));
    }
    Ok(())
}

fn inputs(step: &RawStep<'_>, service: &str) -> Result<(usize, usize), SeedError> {
    let bytes = metadata(step, service)?;
    step.inputs
        .iter()
        .try_fold((0, bytes), |(nodes, bytes), (name, value)| {
            let (count, size) = value::measure(value)?;
            Ok((
                add(nodes, count, 1)?,
                add(add(bytes, name.len(), 1)?, size, 1)?,
            ))
        })
}

fn relationships(document: &Document<'_>) -> Result<(usize, usize), SeedError> {
    document
        .relationships
        .iter()
        .try_fold((0, 0), |(count, bytes), rel| {
            let source = document
                .steps
                .iter()
                .find(|step| step.id == rel.source)
                .ok_or(SeedError::Invalid("seed relationship source is unknown"))?;
            let size = add(rel.relation.len(), rel.observed.len(), 1)?;
            Ok((
                add(count, 1, source.repeat)?,
                add(bytes, size, source.repeat)?,
            ))
        })
}

pub(super) fn add(total: usize, count: usize, repeat: u16) -> Result<usize, SeedError> {
    count
        .checked_mul(usize::from(repeat))
        .and_then(|count| total.checked_add(count))
        .ok_or(SeedError::Invalid("seed expansion count overflow"))
}

fn metadata(step: &RawStep<'_>, service: &str) -> Result<usize, SeedError> {
    let size = [
        service,
        step.id,
        step.operation,
        step.kind,
        step.identity.map_or("", |value| value),
    ]
    .iter()
    .try_fold(0, |total, value| add(total, value.len(), 1))?;
    step.checks.iter().try_fold(size, |total, (name, value)| {
        let text = value
            .as_str()
            .ok_or(SeedError::Invalid("seed check must name an input path"))?;
        add(add(total, name.len(), 1)?, text.len(), 1)
    })
}
