use super::emission::{self, Origin, Terminal};
use super::super::RuntimeCatalogCompileError;
use crate::runtime_catalog::emission_sources;
use crate::overlay::ResourceProjection;
use inventory_core::catalog::{Operation, Service, Target};
use crate::runtime_catalog::model_path::validation_context::ValidationContext;

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
    let row = find_kind_operation_row(context, kind)?;
    let service_name = row.service.to_owned();
    let operation_name = row.operation.to_owned();
    let checked = get_callable_operation(context, row, &service_name, &operation_name, path)?;
    let (service, operation) = operation_for(context, &row.service, &row.operation)?;
    let target = item_target_for_selector(service, operation, checked.plan().items.as_ref())?;
    if path.is_empty() {
        return map_string_target_error(kind, service, target, path);
    }
    match super::optional_resolve(service, target, path) {
        Ok(Some(resolved)) => map_string_target_error(kind, service, resolved, path),
        Ok(None) => Err(kind_path_unresolved(&service_name, &operation_name, path)),
        Err(RuntimeCatalogCompileError::Path { .. }) => {
            Err(kind_path_unresolved(&service_name, &operation_name, path))
        }
        Err(error) => Err(error),
    }
}

fn find_kind_operation_row(
    context: &ValidationContext<'_, '_>,
    kind: &str,
) -> Result<&crate::kind_map::RuntimeOperation, RuntimeCatalogCompileError> {
    context
        .kinds
        .operations()
        .iter()
        .find(|row| row.kind == kind)
        .ok_or_else(|| RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: String::new(),
            reason: "no operation row produces this kind",
        })
}

fn get_callable_operation(
    context: &ValidationContext<'_, '_>,
    row: &crate::kind_map::RuntimeOperation,
    service_name: &str,
    operation_name: &str,
    path: &str,
) -> Result<&crate::overlay::CallableOperation, RuntimeCatalogCompileError> {
    operation_for_checked(context, &row.service, &row.operation).map_err(|_| {
        RuntimeCatalogCompileError::CapabilityContext {
            service: service_name.to_owned(),
            operation: operation_name.to_owned(),
            path: path.to_owned(),
        }
    })
}

fn kind_path_unresolved(service: &str, operation: &str, path: &str) -> RuntimeCatalogCompileError {
    RuntimeCatalogCompileError::CapabilityContext {
        service: service.to_owned(),
        operation: operation.to_owned(),
        path: path.to_owned(),
    }
}

fn map_string_target_error(
    kind: &str,
    service: Service<'_>,
    target: Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    super::string_target(service, target).map_err(|error| match error {
        RuntimeCatalogCompileError::Path { .. } => {
            RuntimeCatalogCompileError::KindContext {
                kind: kind.to_owned(),
                path: path.to_owned(),
                reason: "path does not end in a string",
            }
        }
        error => error,
    })
}

fn validate_kind_origin_path(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
    origin: Origin,
    terminal: Terminal,
) -> Result<(), RuntimeCatalogCompileError> {
    let found = emission_sources::sources(context.kinds, context.checked)
        .filter(|source| source.emits_kind(kind))
        .try_fold(false, |found, source| {
            let (service, operation) = operation_for(context, source.service, source.operation)?;
            let target = match origin {
                Origin::SelectedOutput => {
                    if matches!(source.projection, Some(ResourceProjection::IamTrustSummary(protected)) if protected.as_str() == path) {
                        return Err(RuntimeCatalogCompileError::KindContext {
                            kind: kind.to_owned(),
                            path: path.to_owned(),
                            reason: "path is a protected projection",
                        });
                    }
                    item_target_for_selector(service, operation, source.items)?
                }
                Origin::Request => operation.input()?,
            };
            let resolved = if path.is_empty() {
                target
            } else {
                let Some(resolved) =
                    super::optional_resolve(service, target, path)?
                else {
                    return Ok(found);
                };
                resolved
            };
            let resolved = match terminal {
                Terminal::TextOrList
                    if super::kind(service, resolved)? == "list" =>
                {
                    super::list_element(service, resolved)?
                }
                Terminal::Text | Terminal::TextOrList => resolved,
            };
            super::string_target(service, resolved)?;
            Ok(true)
        })?;
    if found {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: path.to_owned(),
            reason: "no callable emission of this kind resolves the path",
        })
    }
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
        .try_for_each(|row| {
            validate_bootstrap(context, &row.service, &row.operation)
        })
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
    opt_in_path
        .map_or(Ok(()), |path| string_target_for_path(service, item, path))
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