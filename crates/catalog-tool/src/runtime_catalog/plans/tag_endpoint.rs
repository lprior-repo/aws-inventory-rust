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
    let path = selector_path(selector);
    let (ids, target) = model_path::resolve(service, root, path.as_str())?;
    let shape = resolve_shape(service, target, path.as_str())?;
    match selector {
        TagEndpointSelector::Map { .. } if shape.kind()? == "map" => {
            map_selector(service, &shape, ids, path.as_str())
        }
        TagEndpointSelector::List { key, value, .. } if shape.kind()? == "list" => list_selector(
            service,
            &shape,
            (key.as_str(), value.as_str()),
            ids,
            path.as_str(),
        ),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        }),
    }
}

fn selector_path(selector: &TagEndpointSelector) -> &crate::overlay::MemberPath {
    match selector {
        TagEndpointSelector::Map { path } | TagEndpointSelector::List { path, .. } => path,
    }
}

fn resolve_shape<'a>(
    service: Service<'a>,
    target: Target,
    path: &str,
) -> Result<inventory_core::catalog::Shape<'a>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => {
            service
                .shape(index)?
                .ok_or_else(|| RuntimeCatalogCompileError::Path {
                    path: path.to_owned(),
                })
        }
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        }),
    }
}
fn map_selector(
    service: Service<'_>,
    shape: &inventory_core::catalog::Shape<'_>,
    ids: Vec<FieldId>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    model_path::string_target(
        service,
        shape
            .map_key()?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: path.to_owned(),
            })?,
    )?;
    model_path::string_target(
        service,
        shape
            .map_value()?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: path.to_owned(),
            })?,
    )?;
    encode::map([(1, Value::Int(1)), (2, path_value(ids))])
}

fn list_selector(
    service: Service<'_>,
    shape: &inventory_core::catalog::Shape<'_>,
    (key, value): (&str, &str),
    ids: Vec<FieldId>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    let item = shape
        .element()?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        })?;
    let (key, key_target) = model_path::resolve(service, item, key)?;
    let (value, value_target) = model_path::resolve(service, item, value)?;
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
            path: path.to_owned(),
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
