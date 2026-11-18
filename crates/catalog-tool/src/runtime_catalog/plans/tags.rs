use super::super::{RuntimeCatalogCompileError, encode, model_path};
use crate::overlay::{ItemSelector, OperationOverlay};
use inventory_core::{
    catalog::{Member, Operation, Service, Shape, Target},
    record::{FieldId, Value},
};

pub(super) fn compile(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let Some(path) = overlay.tags.as_ref() else {
        return Ok(None);
    };
    let item = selected_item(service, operation, overlay.items.as_ref())?;
    let (fields, target) = if path.as_str() == "$" {
        (Vec::new(), item)
    } else {
        resolve(service, item, path.as_str())?
    };
    let path = Value::List(fields.into_iter().map(field_value).collect());
    let shape = shape(service, target)?;
    match shape.kind()? {
        "list" => list(service, shape, path).map(Some),
        "map" => {
            string(service, shape.map_key()?)?;
            string(service, shape.map_value()?)?;
            encode::map([(1, Value::Int(1)), (2, path)]).map(Some)
        }
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn selected_item(
    service: Service<'_>,
    operation: Operation<'_>,
    selector: Option<&ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    super::super::selection::target(service, operation.output()?, selector)
}

fn resolve(
    service: Service<'_>,
    target: Target,
    path: &str,
) -> Result<(Vec<FieldId>, Target), RuntimeCatalogCompileError> {
    let (fields, terminal) = model_path::resolve(service, target, path)?;
    fields.iter().try_fold(target, |target, id| {
        let parent = shape(service, target)?;
        if parent.kind()? != "structure" {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        parent
            .members()?
            .try_fold(None, |found, member| {
                let member = member?;
                Ok::<_, RuntimeCatalogCompileError>(if member.id == *id {
                    Some(member.target)
                } else {
                    found
                })
            })?
            .ok_or(RuntimeCatalogCompileError::Capability)
    })?;
    Ok((fields, terminal))
}

fn list(
    service: Service<'_>,
    tags: Shape<'_>,
    path: Value,
) -> Result<Value, RuntimeCatalogCompileError> {
    let element = shape(
        service,
        tags.element()?
            .ok_or(RuntimeCatalogCompileError::Capability)?,
    )?;
    if element.kind()? != "structure" {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let key = tag_member(element, &["Key", "key", "TagKey", "tagKey"])?;
    let value = tag_member(element, &["Value", "value", "TagValue", "tagValue"])?;
    string(service, Some(key.target))?;
    string(service, Some(value.target))?;
    encode::map([
        (1, Value::Int(0)),
        (2, path),
        (3, field_value(key.id)),
        (4, field_value(value.id)),
    ])
}

fn tag_member<'a>(
    shape: Shape<'a>,
    names: &[&str],
) -> Result<Member<'a>, RuntimeCatalogCompileError> {
    shape
        .members()?
        .try_fold(None, |found, member| {
            let member = member?;
            if !names.contains(&member.name()?) {
                return Ok(found);
            }
            if found.is_some() {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            Ok(Some(member))
        })?
        .ok_or(RuntimeCatalogCompileError::Capability)
}

fn string(service: Service<'_>, target: Option<Target>) -> Result<(), RuntimeCatalogCompileError> {
    model_path::string_target(
        service,
        target.ok_or(RuntimeCatalogCompileError::Capability)?,
    )
}

fn shape<'a>(
    service: Service<'a>,
    target: Target,
) -> Result<Shape<'a>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(RuntimeCatalogCompileError::Capability),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn field_value(field: FieldId) -> Value {
    Value::Int(i64::from(field.value()))
}
