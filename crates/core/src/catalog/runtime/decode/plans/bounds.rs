use super::super::RuntimeCatalogError;
use crate::record::ValueRef;

pub(super) fn optional_u32(
    map_value: crate::record::MapRef<'_>,
    key: u32,
) -> Result<Option<u32>, RuntimeCatalogError> {
    match map_value.get_field(crate::record::FieldId::new(key)?)? {
        Some(ValueRef::Int(value)) => Ok(Some(positive_u32(value)?)),
        Some(ValueRef::Null) | None => Ok(None),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn positive_u32(value: i64) -> Result<u32, RuntimeCatalogError> {
    u32::try_from(value)
        .ok()
        .filter(|number| *number > 0)
        .ok_or(RuntimeCatalogError::Capability)
}
