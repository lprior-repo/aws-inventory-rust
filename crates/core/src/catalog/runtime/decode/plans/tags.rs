use super::{ItemPlan, RuntimeCatalogError, metadata};
use crate::{
    catalog::{Member, Operation, Service, Shape, Target},
    catalog::runtime::TagPlan,
    record::{FieldId, MapRef},
};

pub(super) fn read(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
    items: &ItemPlan,
) -> Result<Option<TagPlan>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(14)?)? else {
        return Ok(None);
    };
    let tags = parse(super::super::map(value)?)?;
    validate(&tags, service, operation, items)?;
    Ok(Some(tags))
}

fn parse(fields: MapRef<'_>) -> Result<TagPlan, RuntimeCatalogError> {
    let path = metadata::selected_path(metadata::list(fields, 2)?)?;
    match metadata::integer(fields, 1)? {
        0 => {
            metadata::only_fields(fields, &[1, 2, 3, 4])?;
            let key = field_id(fields, 3)?;
            let value = field_id(fields, 4)?;
            if key == value {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(TagPlan::List { path, key, value })
        }
        1 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(TagPlan::Map { path })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn field_id(fields: MapRef<'_>, field: u32) -> Result<FieldId, RuntimeCatalogError> {
    let raw = metadata::integer(fields, field)?;
    let number = u32::try_from(raw).map_err(|_| RuntimeCatalogError::Capability)?;
    Ok(FieldId::new(number)?)
}

fn validate(
    tags: &TagPlan,
    service: Service<'_>,
    operation: Operation<'_>,
    items: &ItemPlan,
) -> Result<(), RuntimeCatalogError> {
    let item = selected_item(service, operation, items)?;
    match tags {
        TagPlan::List { path, key, value } => {
            let tags = shape(service, resolve(service, item, path)?)?;
            if tags.kind()? != "list" {
                return Err(RuntimeCatalogError::Capability);
            }
            let element = shape(service, tags.element()?.ok_or(RuntimeCatalogError::Capability)?)?;
            string(service, Some(member(element, *key)?.target))?;
            string(service, Some(member(element, *value)?.target))
        }
        TagPlan::Map { path } => {
            let tags = shape(service, resolve(service, item, path)?)?;
            if tags.kind()? != "map" {
                return Err(RuntimeCatalogError::Capability);
            }
            string(service, tags.map_key()?)?;
            string(service, tags.map_value()?)
        }
    }
}

pub(super) fn selected_item(
    service: Service<'_>,
    operation: Operation<'_>,
    items: &ItemPlan,
) -> Result<Target, RuntimeCatalogError> {
    super::selection::validate_item(service, operation, items)
}

pub(super) fn resolve(
    service: Service<'_>,
    target: Target,
    path: &[FieldId],
) -> Result<Target, RuntimeCatalogError> {
    if path.len() > 8 {
        return Err(RuntimeCatalogError::Bounds);
    }
    path.iter().try_fold(target, |target, id| {
        Ok(member(shape(service, target)?, *id)?.target)
    })
}

pub(super) fn member<'a>(shape: Shape<'a>, id: FieldId) -> Result<Member<'a>, RuntimeCatalogError> {
    if !matches!(shape.kind()?, "structure" | "union") {
        return Err(RuntimeCatalogError::Capability);
    }
    shape.members()?.try_fold(None, |found, candidate| {
        let candidate = candidate?;
        if candidate.id != id {
            return Ok(found);
        }
        if found.is_some() {
            return Err(RuntimeCatalogError::Capability);
        }
        Ok(Some(candidate))
    })?.ok_or(RuntimeCatalogError::Capability)
}

fn string(service: Service<'_>, target: Option<Target>) -> Result<(), RuntimeCatalogError> {
    if target_kind(service, target.ok_or(RuntimeCatalogError::Capability)?)? != "string" {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

pub(super) fn target_kind(service: Service<'_>, target: Target) -> Result<&'static str, RuntimeCatalogError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        Target::Shape(_) => Ok(shape(service, target)?.kind()?),
    }
}

pub(super) fn shape<'a>(service: Service<'a>, target: Target) -> Result<Shape<'a>, RuntimeCatalogError> {
    match target {
        Target::Shape(index) => service.shape(index)?.ok_or(RuntimeCatalogError::Capability),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

#[cfg(test)]
mod tests;
