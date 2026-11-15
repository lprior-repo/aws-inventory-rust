use super::super::{RuntimeCatalogCompileError, encode, model_path};
use crate::overlay::{ItemSelector, OperationOverlay, ResourceProjection};
use inventory_core::{catalog::{Operation, Service, Target}, inventory::Kind, record::{FieldId, Value}};

pub(super) fn compile(service: Service<'_>, operation: Operation<'_>, overlay: &OperationOverlay) -> Result<Vec<(u32, Value)>, RuntimeCatalogCompileError> {
    let item = selected_item(service, operation, overlay.items.as_ref())?;
    let projection = overlay.projection.as_ref()
        .map(|projection| compile_projection(service, item, projection).map(|value| (17, value)))
        .transpose()?;
    let classification = overlay.classification.as_ref().map(|classification| {
        if classification.variants.is_empty() || classification.variants.len() > 64 {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        let (fields, target) = model_path::resolve(service, item, classification.field.as_str())?;
        model_path::string_target(service, target)?;
        let variants = classification.variants.iter().map(|(value, kind)| {
            if value.is_empty() || value.len() > 256 || !value.is_ascii() {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            Kind::parse(kind).map_err(|_| RuntimeCatalogCompileError::Capability)?;
            encode::map([(1, Value::Text(value.clone())), (2, Value::Text(kind.clone()))])
        }).collect::<Result<Vec<_>, _>>()?;
        encode::map([(1, path_value(fields)), (2, Value::List(variants))]).map(|value| (18, value))
    }).transpose()?;
    Ok(projection.into_iter().chain(classification).collect())
}

fn compile_projection(service: Service<'_>, item: Target, projection: &ResourceProjection) -> Result<Value, RuntimeCatalogCompileError> {
    let (path, mode) = match projection {
        ResourceProjection::IamTrustSummary(path) => (path, 0),
        ResourceProjection::SensitiveDigest(path) => (path, 1),
        ResourceProjection::SensitiveMapDigest { path, .. } => (path, 2),
    };
    let (fields, target) = model_path::resolve(service, item, path.as_str())?;
    let extra = match projection {
        ResourceProjection::SensitiveMapDigest { key, .. } => {
            if key.is_empty() || key.len() > 256 { return Err(RuntimeCatalogCompileError::Capability); }
            let Target::Shape(index) = target else { return Err(RuntimeCatalogCompileError::Capability); };
            let shape = service.shape(index)?.ok_or(RuntimeCatalogCompileError::Capability)?;
            if shape.kind()? != "map" { return Err(RuntimeCatalogCompileError::Capability); }
            model_path::string_target(service, shape.map_value()?.ok_or(RuntimeCatalogCompileError::Capability)?)?;
            Some((3, Value::Text(key.clone())))
        }
        _ => { model_path::string_target(service, target)?; None }
    };
    encode::map([(1, Value::Int(mode)), (2, path_value(fields))].into_iter().chain(extra))
}

fn selected_item(service: Service<'_>, operation: Operation<'_>, selector: Option<&ItemSelector>) -> Result<Target, RuntimeCatalogCompileError> {
    super::super::selection::target(service, operation.output()?, selector)
}
fn path_value(fields: Vec<FieldId>) -> Value {
    Value::List(fields.into_iter().map(|field| Value::Int(i64::from(field.value()))).collect())
}
