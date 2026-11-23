use super::super::{RuntimeCatalogCompileError, ServiceIndex};
use inventory_core::catalog::{Operation, Service};

pub(in crate::runtime_catalog) fn catalog_service<'a>(
    index: &ServiceIndex<'a>,
    key: &str,
) -> Result<Service<'a>, RuntimeCatalogCompileError> {
    index.get(key)
}

pub(in crate::runtime_catalog) fn catalog_operation<'a>(
    service: Service<'a>,
    name: &str,
) -> Result<Operation<'a>, RuntimeCatalogCompileError> {
    service
        .operations()?
        .try_fold(None, |found, operation| {
            let operation = operation?;
            if operation.name()? != name {
                return Ok(found);
            }
            if found.is_some() {
                return Err(inventory_core::catalog::CatalogError::Reference);
            }
            Ok(Some(operation))
        })?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: format!("operation {name}"),
        })
}
