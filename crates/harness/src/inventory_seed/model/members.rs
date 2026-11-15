use super::*;

pub(super) fn field_path(
    service: Service<'_>,
    mut target: Target,
    names: &[&str],
) -> Result<Vec<FieldId>, SeedError> {
    names.iter().try_fold(Vec::new(), |mut path, name| {
        let found = member(service, target, name)?;
        target = found.target;
        path.push(found.id);
        Ok(path)
    })
}

pub(super) fn path_target(
    service: Service<'_>,
    target: Target,
    path: &FieldPath,
) -> Result<Target, SeedError> {
    path.parts().iter().try_fold(target, |target, name| Ok(member(service, target, name)?.target))
}

pub(in crate::inventory_seed) fn member<'a>(
    service: Service<'a>,
    target: Target,
    name: &str,
) -> Result<Member<'a>, SeedError> {
    let Target::Shape(index) = target else {
        return Err(SeedError::Invalid("modeled field path is not a structure"));
    };
    let shape = service
        .shape(index)?
        .ok_or(SeedError::Invalid("modeled shape is missing"))?;
    shape
        .members()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? == name {
                Ok::<Option<Member<'a>>, inventory_core::catalog::CatalogError>(Some(candidate))
            } else {
                Ok(found)
            }
        })?
        .ok_or(SeedError::Invalid("modeled field path is missing"))
}
