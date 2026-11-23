use super::{expand::Bindings, *};

pub(super) fn convert(
    service: &str,
    raw: &RawStep<'_>,
    id: StepId,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Result<SeedStep, SeedError> {
    let inputs = inputs(raw, bindings, position, index)?;
    let checks = checks(raw)?;
    let step = match raw.identity {
        Some(identity) => SeedStep::new(
            id,
            service.into(),
            raw.operation.into(),
            raw.kind.into(),
            path(identity)?,
            inputs,
        ),
        None => SeedStep::auxiliary(id, service.into(), raw.operation.into(), inputs),
    };
    Ok(step.with_response_checks(checks).with_scope(raw.scope))
}

fn inputs(
    raw: &RawStep<'_>,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Result<Vec<SeedInput>, SeedError> {
    raw.inputs
        .iter()
        .map(|(name, input)| {
            Ok(SeedInput::new(
                path(name)?,
                value::convert(input, bindings, position, index)?,
            ))
        })
        .collect()
}

fn checks(raw: &RawStep<'_>) -> Result<Vec<SeedOutputCheck>, SeedError> {
    raw.checks
        .iter()
        .map(|(output, input)| {
            let input = input
                .as_str()
                .ok_or(SeedError::Invalid("seed check must name an input path"))?;
            Ok(SeedOutputCheck::new(path(output)?, path(input)?))
        })
        .collect()
}
