mod identity;
use super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{ItemSelector, SelectionKind};
pub(super) use identity::validate as validate_identity;
use inventory_core::catalog::{Service, Target};

pub(crate) fn target(
    service: Service<'_>,
    output: Target,
    selector: Option<&ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    if let Some(ItemSelector::Nested(nested)) = selector {
        return nested_target(service, output, nested);
    }
    let (path, kind) = match selector {
        Some(ItemSelector::Root) => return Ok(output),
        Some(ItemSelector::Member(path)) => {
            let path_str = path.as_str();
            let selected = model_path::resolve(service, output, path_str)?.1;
            return match model_path::kind(service, selected)? {
                "structure" => Ok(selected),
                "list" => model_path::list_element(service, selected),
                _ => Err(RuntimeCatalogCompileError::Path {
                    path: path_str.to_owned(),
                }),
            };
        }
        Some(ItemSelector::Explicit { path, kind }) => (path, kind),
        Some(ItemSelector::Nested(nested)) => return nested_target(service, output, nested),
        None => {
            return Err(RuntimeCatalogCompileError::Path {
                path: "item selector".to_owned(),
            });
        }
    };
    let path_str = path.as_str();
    let selected = if path_str == "$" {
        output
    } else {
        model_path::resolve(service, output, path_str)?.1
    };
    match (kind, model_path::kind(service, selected)?) {
        (SelectionKind::Structure, "structure") => Ok(selected),
        (
            SelectionKind::Scalar,
            "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean",
        ) => Ok(selected),
        (SelectionKind::List, "list") => model_path::list_element(service, selected),
        (SelectionKind::MapKeys, "map") => {
            shape(service, selected)?
                .map_key()?
                .ok_or_else(|| RuntimeCatalogCompileError::Path {
                    path: path_str.to_owned(),
                })
        }
        (SelectionKind::MapEntries | SelectionKind::MapValues, "map") => shape(service, selected)?
            .map_value()?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: path_str.to_owned(),
            }),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path_str.to_owned(),
        }),
    }
}
fn nested_target(
    service: Service<'_>,
    output: Target,
    nested: &crate::overlay::NestedItems,
) -> Result<Target, RuntimeCatalogCompileError> {
    let parent_list = model_path::resolve(service, output, nested.parents.as_str())?.1;
    if model_path::kind(service, parent_list)? != "list" {
        return Err(RuntimeCatalogCompileError::Path {
            path: nested.parents.to_string(),
        });
    }
    let parent = model_path::list_element(service, parent_list)?;
    let item_list = model_path::resolve(service, parent, nested.items.as_str())?.1;
    let owner = model_path::resolve(service, parent, nested.owner.as_str())?.1;
    if model_path::kind(service, item_list)? != "list"
        || model_path::kind(service, owner)? != "string"
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: nested.parents.to_string(),
        });
    }
    model_path::list_element(service, item_list)
}

fn shape<'a>(
    service: Service<'a>,
    target: Target,
) -> Result<inventory_core::catalog::Shape<'a>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(RuntimeCatalogCompileError::Path {
                path: "shape".to_owned(),
            }),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "shape".to_owned(),
        }),
    }
}
