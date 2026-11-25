use super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{Binding, Identity, ItemSelector, OperationOverlay};
use inventory_core::catalog::{Operation, Service, Target};
mod pagination;
mod resource;
pub(in crate::runtime_catalog) use resource::validate_resource;

pub(super) fn validate(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let output = operation.output()?;
    let input = operation.input()?;
    let item = item_target(service, output, overlay.items.as_ref())?;
    if overlay.emission == crate::overlay::Emission::EmitMetadata
        && overlay.tag_endpoint.is_none()
        && !overlay.endpoint_discovery
    {
        validate_identity(service, operation, item, overlay.identity.as_ref())?;
    }
    validate_bindings(service, input, overlay)?;
    validate_page(service, operation, overlay)?;
    overlay
        .exclude_from_hash
        .iter()
        .try_for_each(|path| model_path::resolve(service, item, &path.to_string()).map(|_| ()))?;
    validate_purpose_paths(service, operation, overlay)?;
    Ok(())
}

fn item_target(
    service: Service<'_>,
    output: Target,
    selector: Option<&ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    super::selection::target(service, output, selector)
}

fn validate_identity(
    service: Service<'_>,
    operation: Operation<'_>,
    item: Target,
    identity: Option<&Identity>,
) -> Result<(), RuntimeCatalogCompileError> {
    match identity {
        Some(Identity::Arn(path)) => validate_identity_arn(service, item, path),
        Some(Identity::Key(paths)) => validate_identity_keys(service, item, paths),
        Some(Identity::Template(template)) => validate_identity_template(service, item, template),
        Some(
            value @ (Identity::Choice(_) | Identity::CorrelatedKey(_) | Identity::MapArn { .. }),
        ) => super::selection::validate_identity(service, operation, item, value),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "identity".to_owned(),
        }),
    }
}

fn validate_identity_arn(
    service: Service<'_>,
    item: Target,
    path: &crate::overlay::MemberPath,
) -> Result<(), RuntimeCatalogCompileError> {
    let path_str = path.as_str();
    let target = map_path_error(path_str, selected_target(service, item, path_str))?;
    map_path_error(path_str, text_target(service, target, path_str))
}

fn validate_identity_keys(
    service: Service<'_>,
    item: Target,
    paths: &[crate::overlay::MemberPath],
) -> Result<(), RuntimeCatalogCompileError> {
    paths.iter().try_for_each(|path| {
        let path_str = path.as_str();
        map_path_error(path_str, scalar_target(service, item, path_str))
    })
}

fn validate_identity_template(
    service: Service<'_>,
    item: Target,
    template: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    super::plans::identity::fields(template).try_for_each(|path| {
        map_path_error(path, scalar_target(service, item, path))
    })
}

fn selected_target(
    service: Service<'_>,
    item: Target,
    path: &str,
) -> Result<Target, RuntimeCatalogCompileError> {
    if path != "$" {
        return model_path::resolve(service, item, path).map(|(_, target)| target);
    }
    match model_path::kind(service, item)? {
        "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean" => Ok(item),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        }),
    }
}

fn text_target(
    service: Service<'_>,
    target: Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    match model_path::kind(service, target)? {
        "string" | "enum" => Ok(()),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        }),
    }
}

fn scalar_target(
    service: Service<'_>,
    item: Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    let target = selected_target(service, item, path)?;
    match model_path::kind(service, target)? {
        "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean" => Ok(()),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        }),
    }
}

fn validate_bindings(
    service: Service<'_>,
    input: Target,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    overlay
        .bind
        .iter()
        .chain(
            overlay
                .branches
                .iter()
                .flat_map(|branch| branch.bind.iter()),
        )
        .try_for_each(|(path, binding)| {
            let target = model_path::resolve(service, input, &path.to_string())?.1;
            let path_str = path.to_string();
            match binding {
                Binding::Region | Binding::Account => {
                    model_path::string_target(service, target).map_err(|_| {
                        RuntimeCatalogCompileError::Path { path: path_str }
                    })
                }
                Binding::RegionList => {
                    let element = model_path::list_element(service, target)?;
                    model_path::string_target(service, element).map_err(|_| {
                        RuntimeCatalogCompileError::Path { path: path_str }
                    })
                }
                Binding::FromAny(alternatives) => {
                    if alternatives.is_empty()
                        || alternatives.len() > 8
                        || alternatives.iter().enumerate().any(|(index, alternative)| {
                            alternatives
                                .iter()
                                .take(index)
                                .any(|prior| prior.operation == alternative.operation)
                        })
                    {
                        return Err(RuntimeCatalogCompileError::Path { path: path_str });
                    }
                    Ok(())
                }
                Binding::From { .. }
                | Binding::Context { .. }
                | Binding::FromArn { .. }
                | Binding::FromUrl { .. }
                | Binding::FromService { .. }
                | Binding::FromBranch { .. }
                | Binding::ContextFromBranch { .. } => Ok(()),
                Binding::Transformed {
                    binding,
                    transform: crate::overlay::BindingTransform::StripPrefix(prefix),
                } => {
                    if prefix.is_empty()
                        || prefix.len() > 256
                        || prefix.chars().any(char::is_control)
                        || !matches!(
                            binding.as_ref(),
                            Binding::From { .. }
                                | Binding::Context { .. }
                                | Binding::FromArn { .. }
                                | Binding::FromService { .. }
                        )
                    {
                        return Err(RuntimeCatalogCompileError::Path { path: path_str });
                    }
                    model_path::string_target(service, target).map_err(|_| {
                        RuntimeCatalogCompileError::Path { path: path_str }
                    })
                }
            }
        })
}

fn validate_page(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    pagination::validate(service, operation, overlay)
}

fn validate_purpose_paths(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    if operation.paginator()?.is_some() && overlay.page_size.is_some() {
        let page_size = operation.paginator()?.and_then(|page| page.page_size);
        let path = page_size.ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "page_size".to_owned(),
        })?;
        model_path::resolve(service, operation.input()?, path)?;
    }
    Ok(())
}

fn map_path_error<T>(
    path: &str,
    result: Result<T, RuntimeCatalogCompileError>,
) -> Result<T, RuntimeCatalogCompileError> {
    result.map_err(|error| match error {
        RuntimeCatalogCompileError::Path { path: inner }
            if inner != path =>
        {
            RuntimeCatalogCompileError::Path { path: path.to_owned() }
        }
        error => error,
    })
}
