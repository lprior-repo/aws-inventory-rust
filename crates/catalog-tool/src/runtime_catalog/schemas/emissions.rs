use super::{Context, RuntimeCatalogCompileError, selected};
use crate::runtime_catalog::{emission_sources, model_path, plans};
use std::collections::BTreeMap;

/// Additional namespaces do not borrow the primary item's field types or columns.
pub(super) fn columns(context: &Context<'_, '_>, kind: &str)
    -> Result<BTreeMap<String, i64>, RuntimeCatalogCompileError> {
    let Some(checked) = context.checked else { return Ok(BTreeMap::new()); };
    emission_sources::sources(context.kinds, checked)
        .filter(|source| source.key != "primary" && source.emits_kind(kind))
        .try_fold(BTreeMap::new(), |mut columns, source| {
            let service = plans::catalog_service(context.services, source.service)?;
            let operation = plans::catalog_operation(service, source.operation)?;
            let item = crate::runtime_catalog::selection::target(service, operation.output()?, source.items)?;
            source.queryable.iter().try_for_each(|path| {
                let target = if path.as_str() == "$" { item } else { model_path::resolve(service, item, path.as_str())?.1 };
                let scalar = selected::scalar_code(model_path::kind(service, target)?)?
                    .ok_or(RuntimeCatalogCompileError::Capability)?;
                if columns.insert(path.as_str().to_owned(), scalar).is_some_and(|previous| previous != scalar) {
                    return Err(RuntimeCatalogCompileError::Capability);
                }
                Ok(())
            })?;
            Ok(columns)
        })
}
