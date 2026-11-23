use super::{OperationOverlay, RuntimeCatalogCompileError, encode};
use inventory_core::record::Value;

pub(super) fn compile(
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if overlay.exclude_response_members.is_empty() {
        return Ok(None);
    }
    if overlay.exclude_response_members.len() > 8 {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let paths = overlay
        .exclude_response_members
        .iter()
        .map(|path| encode::path(services, path.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some((24, Value::List(paths))))
}
