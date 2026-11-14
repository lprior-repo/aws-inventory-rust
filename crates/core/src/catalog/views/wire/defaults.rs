use super::{CatalogError, CatalogKey, ValueRef, WireRole, WireTraits};

pub(super) fn default_value<'a>(
    wire: WireTraits<'a>,
) -> Result<Option<ValueRef<'a>>, CatalogError> {
    let Some(map) = wire.map.filter(|_| matches!(wire.role, WireRole::Member)) else {
        return Ok(None);
    };
    match map.get_field(CatalogKey::DefaultValue.field()?)? {
        None => Ok(None),
        Some(value) => Ok(Some(value)),
    }
}

pub(super) fn client_optional(wire: WireTraits<'_>) -> Result<bool, CatalogError> {
    match wire.role {
        WireRole::Member => wire.flag(CatalogKey::ClientOptional),
        WireRole::Empty | WireRole::Shape | WireRole::Service => Ok(false),
    }
}
