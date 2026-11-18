use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::inventory_seed) enum Selector {
    Field(FieldId),
    Name(String),
    Index(usize),
}

pub(in crate::inventory_seed) fn output_path(
    service: Service<'_>,
    target: Target,
    path: &FieldPath,
) -> Result<Vec<Selector>, SeedError> {
    resolve_path(service, target, path).map(|(path, _)| path)
}

fn resolve_path(
    service: Service<'_>,
    target: Target,
    path: &FieldPath,
) -> Result<(Vec<Selector>, Target), SeedError> {
    path.parts()
        .iter()
        .try_fold((Vec::new(), target), |(mut path, target), part| {
            let (selector, target) = select_target(service, target, part)?;
            path.push(selector);
            Ok((path, target))
        })
}

pub(super) fn text_path(
    service: Service<'_>,
    target: Target,
    path: &FieldPath,
) -> Result<Vec<Selector>, SeedError> {
    let (path, target) = resolve_path(service, target, path)?;
    text_target(service, target)?;
    Ok(path)
}

pub(super) fn text_target(service: Service<'_>, target: Target) -> Result<(), SeedError> {
    let kind = match target {
        Target::Prelude(kind) => kind,
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(SeedError::Invalid("seed identity shape is missing"))?
            .kind()?,
        _ => return Err(SeedError::Invalid("seed identity target is unknown")),
    };
    if !matches!(kind, "string" | "enum") {
        return Err(SeedError::Invalid(
            "seed identity path must select modeled text",
        ));
    }
    Ok(())
}

pub(super) fn select_target(
    service: Service<'_>,
    target: Target,
    part: &str,
) -> Result<(Selector, Target), SeedError> {
    let Target::Shape(index) = target else {
        return Err(SeedError::Invalid(
            "seed output selector does not address a container",
        ));
    };
    let shape = service
        .shape(index)?
        .ok_or(SeedError::Invalid("seed output shape is missing"))?;
    match shape.kind()? {
        "structure" | "union" => {
            let member = members::member(service, target, part)?;
            Ok((Selector::Field(member.id), member.target))
        }
        "map" => Ok((
            Selector::Name(part.to_owned()),
            shape
                .map_value()?
                .ok_or(SeedError::Invalid("seed output map value shape is missing"))?,
        )),
        "list" => list_selector(shape, part),
        _ => Err(SeedError::Invalid(
            "seed output selector does not address a container",
        )),
    }
}

fn list_selector(
    shape: inventory_core::catalog::Shape<'_>,
    part: &str,
) -> Result<(Selector, Target), SeedError> {
    let index = part
        .parse::<usize>()
        .ok()
        .filter(|index| *index < 256)
        .ok_or(SeedError::Invalid(
            "seed list selector must be an index below 256",
        ))?;
    Ok((
        Selector::Index(index),
        shape.element()?.ok_or(SeedError::Invalid(
            "seed output list element shape is missing",
        ))?,
    ))
}
