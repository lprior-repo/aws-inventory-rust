use super::{OperationOverlay, RuntimeCatalogCompileError, encode};
use inventory_core::record::Value;

#[cfg(test)]
mod tests;

const MAX_EXCLUDED_MEMBERS: usize = 8;

pub(super) fn compile(
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if overlay.exclude_response_members.is_empty() {
        return Ok(None);
    }
    bounded(overlay)?;
    let paths = overlay
        .exclude_response_members
        .iter()
        .map(|path| encode::path(services, path.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some((24, Value::List(paths))))
}

fn bounded(overlay: &OperationOverlay) -> Result<(), RuntimeCatalogCompileError> {
    if overlay.exclude_response_members.len() > MAX_EXCLUDED_MEMBERS {
        return Err(RuntimeCatalogCompileError::Path {
            path: "exclude_response_members".to_owned(),
        });
    }
    Ok(())
}
