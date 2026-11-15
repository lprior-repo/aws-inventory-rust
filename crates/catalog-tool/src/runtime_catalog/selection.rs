mod identity;
pub(super) use identity::validate as validate_identity;
use super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{ItemSelector, SelectionKind};
use inventory_core::catalog::{Service, Target};

pub(super) fn target(service: Service<'_>, output: Target, selector: Option<&ItemSelector>) -> Result<Target, RuntimeCatalogCompileError> {
    let (path, kind) = match selector {
        Some(ItemSelector::Root) => return Ok(output),
        Some(ItemSelector::Member(path)) => {
            let selected = model_path::resolve(service, output, path.as_str())?.1;
            return match model_path::kind(service, selected)? {
                "structure" => Ok(selected), "list" => model_path::list_element(service, selected),
                _ => Err(RuntimeCatalogCompileError::Capability),
            };
        }
        Some(ItemSelector::Explicit { path, kind }) => (path, kind),
        None => return Err(RuntimeCatalogCompileError::Capability),
    };
    let selected = if path.as_str() == "$" { output } else { model_path::resolve(service, output, path.as_str())?.1 };
    match (kind, model_path::kind(service, selected)?) {
        (SelectionKind::Structure, "structure") => Ok(selected),
        (SelectionKind::Scalar, "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean") => Ok(selected),
        (SelectionKind::List, "list") => model_path::list_element(service, selected),
        (SelectionKind::MapKeys, "map") => shape(service, selected)?.map_key()?.ok_or(RuntimeCatalogCompileError::Capability),
        (SelectionKind::MapEntries | SelectionKind::MapValues, "map") => shape(service, selected)?.map_value()?.ok_or(RuntimeCatalogCompileError::Capability),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn shape<'a>(service: Service<'a>, target: Target) -> Result<inventory_core::catalog::Shape<'a>, RuntimeCatalogCompileError> {
    match target { Target::Shape(index) => service.shape(index)?.ok_or(RuntimeCatalogCompileError::Capability),
        _ => Err(RuntimeCatalogCompileError::Capability) }
}
