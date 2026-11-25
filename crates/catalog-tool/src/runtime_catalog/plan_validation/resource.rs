use super::{
    Operation, OperationOverlay, RuntimeCatalogCompileError, Service, item_target, model_path,
    selected_target, validate_identity,
};

pub(in crate::runtime_catalog) fn validate_resource(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let item = item_target(service, operation.output()?, overlay.items.as_ref())?;
    validate_identity(service, operation, item, overlay.identity.as_ref())?;
    overlay
        .exclude_from_hash
        .iter()
        .try_for_each(|path| model_path::resolve(service, item, path.as_str()).map(|_| ()))?;
    overlay.queryable.iter().try_for_each(|path| {
        let target = selected_target(service, item, path.as_str())?;
        match model_path::kind(service, target)? {
            "boolean" | "byte" | "short" | "integer" | "long" | "float" | "double" | "string"
            | "enum" | "blob" | "timestamp" => Ok(()),
            kind => Err(RuntimeCatalogCompileError::Path {
                path: format!("queryable path kind {kind}"),
            }),
        }
    })
}
