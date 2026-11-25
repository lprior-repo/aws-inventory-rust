use super::super::RuntimeCatalogCompileError;
use super::emission::{self, Origin, Terminal};
use super::validation_context::ValidationContext;
use inventory_core::catalog::{Operation, Service};

mod origin;
use origin::validate as validate_kind_origin_path;

mod relations;

pub fn validate_reviewed_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    validate_display_paths(context)?;
    relations::validate_relations(context)?;
    validate_bootstrap_paths(context)
}

fn validate_display_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context.kinds.entries().iter().try_for_each(|entry| {
        match (&entry.target, entry.display_name_path.as_deref()) {
            (crate::kind_map::KindTarget::Mapped { kind, .. }, Some(path))
                if emission::active_kind(context, kind) =>
            {
                validate_kind_path(context, kind, path)
            }
            _ => Ok(()),
        }
    })
}

fn validate_kind_path(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    validate_kind_origin_path(context, kind, path, Origin::SelectedOutput, Terminal::Text)
}

fn validate_bootstrap_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context
        .kinds
        .operations()
        .iter()
        .filter(|row| {
            operation_for_checked(context, &row.service, &row.operation).is_ok_and(|call| {
                let plan = call.plan();
                plan.emission == crate::overlay::Emission::EmitMetadata
                    && plan.tag_endpoint.is_none()
                    && !plan.endpoint_discovery
            })
        })
        .try_for_each(|row| validate_bootstrap(context, &row.service, &row.operation))
}

fn validate_bootstrap(
    context: &ValidationContext<'_, '_>,
    service_name: &str,
    operation_name: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    match context.kinds.bootstrap_for(service_name, operation_name) {
        Some(crate::kind_map::BootstrapPurpose::AccountIdentity { account_path }) => {
            let (service, operation) = operation_for(context, service_name, operation_name)?;
            string_target_for_path(service, operation.output()?, account_path)
        }
        Some(crate::kind_map::BootstrapPurpose::RegionDiscovery {
            regions_path,
            region_name_path,
            opt_in_status_path,
        }) => validate_region_paths(
            context,
            service_name,
            operation_name,
            regions_path,
            region_name_path,
            opt_in_status_path.as_deref(),
        ),
        Some(crate::kind_map::BootstrapPurpose::Enumeration) | None => Ok(()),
    }
}

fn validate_region_paths(
    context: &ValidationContext<'_, '_>,
    service_name: &str,
    operation_name: &str,
    regions_path: &str,
    name_path: &str,
    opt_in_path: Option<&str>,
) -> Result<(), RuntimeCatalogCompileError> {
    let (service, operation) = operation_for(context, service_name, operation_name)?;
    let item = super::list_element(
        service,
        super::resolve(service, operation.output()?, regions_path)?.1,
    )?;
    string_target_for_path(service, item, name_path)?;
    opt_in_path.map_or(Ok(()), |path| string_target_for_path(service, item, path))
}

pub fn item_target_for_selector(
    service: Service<'_>,
    operation: Operation<'_>,
    selector: Option<&crate::overlay::ItemSelector>,
) -> Result<inventory_core::catalog::Target, RuntimeCatalogCompileError> {
    crate::runtime_catalog::selection::target(service, operation.output()?, selector)
}

pub fn string_target_for_path(
    service: Service<'_>,
    target: inventory_core::catalog::Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    super::string_target(service, super::resolve(service, target, path)?.1)
}

pub fn operation_for<'a>(
    context: &ValidationContext<'_, 'a>,
    service: &str,
    operation: &str,
) -> Result<(Service<'a>, Operation<'a>), RuntimeCatalogCompileError> {
    let service_view = context.services.get(service)?;
    let checked_operation = operation_for_checked(context, service, operation)?;
    Ok((
        service_view,
        crate::runtime_catalog::plans::catalog_operation(
            service_view,
            checked_operation.operation(),
        )?,
    ))
}

pub fn operation_for_checked<'p, 'a>(
    context: &ValidationContext<'p, 'a>,
    service: &str,
    operation: &str,
) -> Result<&'p crate::overlay::CallableOperation, RuntimeCatalogCompileError> {
    context
        .checked
        .services()
        .iter()
        .find(|review| review.service() == service)
        .and_then(|review| {
            review
                .callable()
                .iter()
                .find(|candidate| candidate.operation() == operation)
        })
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: format!("{service}/{operation}"),
        })
}

#[cfg(test)]
mod tests;
