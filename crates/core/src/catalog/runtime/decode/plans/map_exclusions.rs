use super::{ItemPlan, RuntimeCatalogError, metadata, tags};
use crate::{
    catalog::{Operation, Service, Target},
    record::{Exclusions, FieldId, MapKeyExclusion, MapRef, ValueRef},
};

pub(super) fn read(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
    items: &ItemPlan,
    exclusions: Exclusions,
) -> Result<Exclusions, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(21)?)? else {
        return Ok(exclusions);
    };
    let ValueRef::List(list) = value else {
        return Err(RuntimeCatalogError::Capability);
    };
    let item = tags::selected_item(service, operation, items)?;
    let masks = metadata::bounded(list, |value| parse(value, service, item), 64)?;
    exclusions.with_map_keys(masks).map_err(|_| RuntimeCatalogError::Capability)
}

fn parse(
    value: ValueRef<'_>,
    service: Service<'_>,
    item: Target,
) -> Result<MapKeyExclusion, RuntimeCatalogError> {
    let ValueRef::Map(fields) = value else {
        return Err(RuntimeCatalogError::Capability);
    };
    metadata::only_fields(fields, &[1, 2])?;
    let path = metadata::id_path(metadata::list(fields, 1)?)?;
    let ValueRef::Text(key) = metadata::field(fields, 2)? else {
        return Err(RuntimeCatalogError::Capability);
    };
    if key.is_empty() {
        return Err(RuntimeCatalogError::Capability);
    }
    if key.len() > 4096 {
        return Err(RuntimeCatalogError::Bounds);
    }
    validate_map(service, tags::resolve(service, item, &path)?)?;
    MapKeyExclusion::new(path.into_vec(), key).map_err(|_| RuntimeCatalogError::Capability)
}

fn validate_map(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    let map = tags::shape(service, target)?;
    if map.kind()? != "map" {
        return Err(RuntimeCatalogError::Capability);
    }
    let key = map.map_key()?.ok_or(RuntimeCatalogError::Capability)?;
    if !matches!(tags::target_kind(service, key)?, "string" | "enum") {
        return Err(RuntimeCatalogError::Capability);
    }
    let value = map.map_value()?.ok_or(RuntimeCatalogError::Capability)?;
    if !matches!(tags::target_kind(service, value)?, "string" | "enum" | "intEnum" | "boolean" | "byte" | "short" | "integer" | "long" | "float" | "double" | "timestamp" | "blob") {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}
