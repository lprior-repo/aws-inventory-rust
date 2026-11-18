use super::{OperationOverlay, RuntimeCatalogCompileError};
use crate::runtime_catalog::{encode, model_path};
use inventory_core::{
    catalog::{Service, Target},
    record::{Exclusions, MapKeyExclusion, Value},
};

pub(super) fn encode(
    service: Service<'_>,
    item: Target,
    overlay: &OperationOverlay,
) -> Result<Value, RuntimeCatalogCompileError> {
    if overlay.exclude_map_keys.len() > 64 {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let masks = overlay
        .exclude_map_keys
        .iter()
        .map(|mask| {
            let (path, target) = model_path::resolve(service, item, mask.path.as_str())?;
            validate_map(service, target)?;
            MapKeyExclusion::new(path, mask.key.as_str())
                .map_err(|_| RuntimeCatalogCompileError::Capability)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let masks = Exclusions::none()
        .with_map_keys(masks)
        .map_err(|_| RuntimeCatalogCompileError::Capability)?;
    masks
        .map_keys()
        .iter()
        .map(|mask| {
            encode::map([
                (
                    1,
                    Value::List(
                        mask.path()
                            .iter()
                            .map(|id| Value::Int(i64::from(id.value())))
                            .collect(),
                    ),
                ),
                (2, Value::Text(mask.key().to_owned())),
            ])
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Value::List)
}

fn validate_map(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    let Target::Shape(index) = target else {
        return Err(RuntimeCatalogCompileError::Capability);
    };
    let map = service
        .shape(index)?
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    if map.kind()? != "map" {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    model_path::string_target(
        service,
        map.map_key()?
            .ok_or(RuntimeCatalogCompileError::Capability)?,
    )?;
    let value = model_path::kind(
        service,
        map.map_value()?
            .ok_or(RuntimeCatalogCompileError::Capability)?,
    )?;
    if !matches!(
        value,
        "string"
            | "enum"
            | "intEnum"
            | "boolean"
            | "byte"
            | "short"
            | "integer"
            | "long"
            | "float"
            | "double"
            | "timestamp"
            | "blob"
    ) {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    Ok(())
}
