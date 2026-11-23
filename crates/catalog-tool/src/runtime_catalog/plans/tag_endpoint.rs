use super::super::{RuntimeCatalogCompileError, encode, model_path};
use crate::overlay::{
    ItemSelector, OperationOverlay, TagEndpointParent, TagEndpointRequest, TagEndpointSelector,
};
use inventory_core::{
    catalog::{Operation, Service, Target},
    record::{FieldId, Value},
};

pub(super) fn compile(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let Some(endpoint) = &overlay.tag_endpoint else {
        return Ok(None);
    };
    if overlay.items != Some(ItemSelector::Root)
        || overlay.tags.is_some()
        || overlay.identity.is_some()
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: "tag endpoint".to_owned(),
        });
    }
    if endpoint.parents.is_empty() || endpoint.parents.len() > 64 {
        return Err(RuntimeCatalogCompileError::Path {
            path: "tag endpoint parents".to_owned(),
        });
    }
    let parents = endpoint
        .parents
        .iter()
        .map(|parent| parent_value(service, operation, parent, services))
        .collect::<Result<Vec<_>, _>>()?;
    let tags = selector(service, operation.output()?, &endpoint.selector)?;
    encode::map([(1, Value::Int(3)), (2, Value::List(parents)), (4, tags)]).map(Some)
}

fn parent_value(
    service: Service<'_>,
    operation: Operation<'_>,
    parent: &TagEndpointParent,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let source = super::catalog_operation(service, parent.operation.as_str())?;
    if source.name()? == operation.name()? {
        return Err(RuntimeCatalogCompileError::Path {
            path: "tag endpoint parent self-reference".to_owned(),
        });
    }
    let request = match &parent.request {
        TagEndpointRequest::None => Value::Null,
        TagEndpointRequest::Owner { input } => {
            let (ids, target) = model_path::resolve(service, operation.input()?, input.as_str())?;
            model_path::string_target(service, target)?;
            encode::map([
                (1, path_value(ids)),
                (2, Value::List(Vec::new())),
                (3, Value::Int(2)),
            ])?
        }
        TagEndpointRequest::Member {
            input,
            parent: source_path,
        }
        | TagEndpointRequest::Context {
            input,
            parent: source_path,
        } => {
            let (ids, target) = model_path::resolve(service, operation.input()?, input.as_str())?;
            model_path::string_target(service, target)?;
            let source_ids = if source_path.as_str() == "$" {
                Value::List(Vec::new())
            } else {
                encode::path(services, source_path.as_str())?
            };
            let origin = match &parent.request {
                TagEndpointRequest::Context { .. } => 1,
                _ => 0,
            };
            encode::map([
                (1, path_value(ids)),
                (2, source_ids),
                (3, Value::Int(origin)),
            ])?
        }
    };
    encode::map([
        (1, Value::Text(parent.operation.as_str().to_owned())),
        (2, request),
    ])
}

fn selector(
    service: Service<'_>,
    root: Target,
    selector: &TagEndpointSelector,
) -> Result<Value, RuntimeCatalogCompileError> {
    let path = match selector {
        TagEndpointSelector::Map { path } | TagEndpointSelector::List { path, .. } => path,
    };
    let (ids, target) = model_path::resolve(service, root, path.as_str())?;
    let shape = match target {
        Target::Shape(index) => {
            service
                .shape(index)?
                .ok_or_else(|| RuntimeCatalogCompileError::Path {
                    path: path.as_str().to_owned(),
                })
        }
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        }),
    }?;
    match selector {
        TagEndpointSelector::Map { .. } if shape.kind()? == "map" => {
            model_path::string_target(
                service,
                shape
                    .map_key()?
                    .ok_or_else(|| RuntimeCatalogCompileError::Path {
                        path: path.as_str().to_owned(),
                    })?,
            )?;
            model_path::string_target(
                service,
                shape
                    .map_value()?
                    .ok_or_else(|| RuntimeCatalogCompileError::Path {
                        path: path.as_str().to_owned(),
                    })?,
            )?;
            encode::map([(1, Value::Int(1)), (2, path_value(ids))])
        }
        TagEndpointSelector::List { key, value, .. } if shape.kind()? == "list" => {
            let item = shape
                .element()?
                .ok_or_else(|| RuntimeCatalogCompileError::Path {
                    path: path.as_str().to_owned(),
                })?;
            let (key, key_target) = model_path::resolve(service, item, key.as_str())?;
            let (value, value_target) = model_path::resolve(service, item, value.as_str())?;
            model_path::string_target(service, key_target)?;
            model_path::string_target(service, value_target)?;
            match (key.as_slice(), value.as_slice()) {
                ([key], [value]) if key != value => encode::map([
                    (1, Value::Int(0)),
                    (2, path_value(ids)),
                    (3, Value::Int(i64::from(key.value()))),
                    (4, Value::Int(i64::from(value.value()))),
                ]),
                _ => Err(RuntimeCatalogCompileError::Path {
                    path: path.as_str().to_owned(),
                }),
            }
        }
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        }),
    }
}

fn path_value(ids: Vec<FieldId>) -> Value {
    Value::List(
        ids.into_iter()
            .map(|field| Value::Int(i64::from(field.value())))
            .collect(),
    )
}
