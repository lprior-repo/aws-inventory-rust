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
        return Err(RuntimeCatalogCompileError::Path {
            path: "exclude_map_keys count".to_owned(),
        });
    }
    let masks = overlay
        .exclude_map_keys
        .iter()
        .map(|mask| {
            let path_str = mask.path.as_str();
            let (path, target) = model_path::resolve(service, item, path_str)?;
            validate_map(service, target)?;
            MapKeyExclusion::new(path, mask.key.as_str()).map_err(|_| {
                RuntimeCatalogCompileError::Path {
                    path: path_str.to_owned(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let masks = Exclusions::none()
        .with_map_keys(masks)
        .map_err(|_| RuntimeCatalogCompileError::Path {
            path: "exclude_map_keys".to_owned(),
        })?;
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
        return Err(RuntimeCatalogCompileError::Path {
            path: "map_exclusions target".to_owned(),
        });
    };
    let map = service
        .shape(index)?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "map_exclusions map".to_owned(),
        })?;
    if map.kind()? != "map" {
        return Err(RuntimeCatalogCompileError::Path {
            path: "map_exclusions kind".to_owned(),
        });
    }
    model_path::string_target(
        service,
        map.map_key()
            ?.ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: "map_exclusions key".to_owned(),
            })?,
    )?;
    let value = model_path::kind(
        service,
        map.map_value()
            ?.ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: "map_exclusions value".to_owned(),
            })?,
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
        return Err(RuntimeCatalogCompileError::Path {
            path: format!("map_exclusions value kind {value}"),
        });
    }
    Ok(())
}
