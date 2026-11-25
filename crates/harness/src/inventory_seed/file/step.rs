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
    let service = raw.service.map_or(service, |value| value).to_owned();
    let step = match raw.identity {
        Some(identity) => SeedStep::new(
            id,
            service,
            raw.operation.into(),
            raw.kind.into(),
            seed_identity(identity)?,
            inputs,
        ),
        None => SeedStep::auxiliary(id, service, raw.operation.into(), inputs),
    };
    Ok(step
        .with_response_checks(checks(raw)?)
        .with_scope(raw.scope))
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

fn seed_identity(value: &toml::Value) -> Result<super::super::SeedIdentity, SeedError> {
    use super::super::SeedIdentity;
    if let Some(value) = value.as_str() {
        return Ok(SeedIdentity::Response(path(value)?));
    }
    let value = descriptor::table(value)?;
    keys(value, &["input", "template"])?;
    let template = value
        .get("template")
        .map(|_| field(value, "template").map(str::to_owned))
        .transpose()?;
    Ok(SeedIdentity::Request {
        input_field: path(field(value, "input")?)?,
        template,
    })
}
