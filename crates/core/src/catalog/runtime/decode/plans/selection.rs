mod identity;
mod validation;
pub(super) use identity::read as identity;
pub(super) use validation::{validate_identity, validate_item, validate_projection};
use super::{metadata, RuntimeCatalogError};
use crate::{catalog::runtime::{BindingSource, ContextProjectionPlan, ContextSource, Emission, ItemPlan, SelectionKind}, record::{FieldId, MapRef, ValueRef}};

pub(super) fn item(fields: MapRef<'_>) -> Result<ItemPlan, RuntimeCatalogError> {
    let tag = metadata::integer(fields, 1)?;
    if tag <= 1 { return super::item_plan(fields) }
    metadata::only_fields(fields, &[1, 2])?;
    let kind = match tag {
        2 => SelectionKind::MapEntries, 3 => SelectionKind::MapKeys,
        4 => SelectionKind::MapValues, 5 => SelectionKind::List,
        6 => SelectionKind::Structure, 7 => SelectionKind::Scalar,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(ItemPlan::Select { path: metadata::selected_path(metadata::list(fields, 2)?)?, kind })
}

pub(super) fn emission(fields: MapRef<'_>) -> Result<Emission, RuntimeCatalogError> {
    match fields.get_field(FieldId::new(19)?)? {
        None | Some(ValueRef::Int(0)) => Ok(Emission::EmitMetadata),
        Some(ValueRef::Int(1)) => Ok(Emission::DiscoveryOnly),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn projection(fields: MapRef<'_>) -> Result<ContextProjectionPlan, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(20)?)? else { return Ok(ContextProjectionPlan::default()) };
    let map = super::map(value)?;
    metadata::only_fields(map, &[1, 2, 3])?;
    Ok(ContextProjectionPlan { selected: paths(map, 1)?, response: paths(map, 2)?, request: paths(map, 3)? })
}

fn paths(map: MapRef<'_>, field: u32) -> Result<Box<[Box<[FieldId]>]>, RuntimeCatalogError> {
    metadata::bounded(metadata::list(map, field)?, |value| {
        metadata::selected_path(super::super::list_value(value)?)
    }, 256).map(Vec::into_boxed_slice)
}

pub(super) fn binding(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
    let source = source(metadata::integer(fields, 2)?)?;
    let service = metadata::text(fields, 3)?;
    let operation = metadata::text(fields, 4)?;
    crate::inventory::ServiceId::parse(&service)?;
    crate::inventory::OperationId::parse(&operation)?;
    Ok(BindingSource::Context { service: service.into_boxed_str(), operation: operation.into_boxed_str(), source,
        path: metadata::selected_path(metadata::list(fields, 5)?)? })
}

pub(super) fn source(tag: i64) -> Result<ContextSource, RuntimeCatalogError> {
    match tag { 5 => Ok(ContextSource::SelectedItem), 6 => Ok(ContextSource::Response), 7 => Ok(ContextSource::Request),
        _ => Err(RuntimeCatalogError::Capability) }
}

pub(super) fn transformed(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4])?;
    let value = metadata::field(fields, 3)?;
    let inner = super::map(value)?;
    if !matches!(metadata::integer(inner, 2)?, 4..=8) {
        return Err(RuntimeCatalogError::Capability);
    }
    let nested = super::binding(value)?;
    if nested.path != metadata::id_path(metadata::list(fields, 1)?)? {
        return Err(RuntimeCatalogError::Capability);
    }
    let prefix = metadata::text(fields, 4)?;
    if prefix.is_empty() || prefix.len() > 256 || prefix.chars().any(char::is_control) {
        return Err(RuntimeCatalogError::Bounds);
    }
    Ok(BindingSource::Transformed {
        source: Box::new(nested.source),
        transform: crate::catalog::runtime::BindingTransform::StripPrefix(prefix.into_boxed_str()),
    })
}

pub(super) fn url_binding(fields: MapRef<'_>) -> Result<BindingSource, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6])?;
    let service = metadata::text(fields, 3)?;
    let operation = metadata::text(fields, 4)?;
    crate::inventory::ServiceId::parse(&service)?;
    crate::inventory::OperationId::parse(&operation)?;
    let resource_kind = match metadata::integer(fields, 6)? {
        0 => crate::catalog::runtime::UrlResourceKind::SqsQueue,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingSource::OperationUrl { service: service.into_boxed_str(), operation: operation.into_boxed_str(),
        path: metadata::selected_path(metadata::list(fields, 5)?)?, resource_kind })
}

pub(super) fn arn_kind(tag: i64) -> Result<crate::catalog::runtime::ArnResourceKind, RuntimeCatalogError> {
    use crate::catalog::runtime::ArnResourceKind;
    match tag { 0 => Ok(ArnResourceKind::SnsSubscription), 1 => Ok(ArnResourceKind::SqsQueue),
        2 => Ok(ArnResourceKind::SnsTopic), _ => Err(RuntimeCatalogError::Capability) }
}
