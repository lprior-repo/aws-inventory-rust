use super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{Binding, Identity, ItemSelector, OperationOverlay, Paginator};
use inventory_core::catalog::{Operation, Service, Target};

pub(super) fn validate(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let output = operation.output()?;
    let input = operation.input()?;
    let item = item_target(service, output, overlay.items.as_ref())?;
    if overlay.emission == crate::overlay::Emission::EmitMetadata && overlay.tag_endpoint.is_none() && !overlay.endpoint_discovery {
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
        Some(Identity::Arn(path)) => {
            let target = selected_target(service, item, path.as_str())?;
            text_target(service, target)
        }
        Some(Identity::Key(paths)) => paths
            .iter()
            .try_for_each(|path| scalar_target(service, item, path.as_str())),
        Some(Identity::Template(template)) => super::plans::identity::fields(template)
            .try_for_each(|path| scalar_target(service, item, path)),
        Some(value @ (Identity::Choice(_) | Identity::CorrelatedKey(_) | Identity::MapArn { .. })) => super::selection::validate_identity(service, operation, item, value),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
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
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn text_target(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    match model_path::kind(service, target)? {
        "string" | "enum" => Ok(()),
        _ => Err(RuntimeCatalogCompileError::Capability),
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
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn validate_bindings(
    service: Service<'_>,
    input: Target,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    overlay.bind.iter().chain(overlay.branches.iter().flat_map(|branch| branch.bind.iter())).try_for_each(|(path, binding)| {
        let target = model_path::resolve(service, input, &path.to_string())?.1;
        match binding {
            Binding::Region | Binding::Account => model_path::string_target(service, target),
            Binding::RegionList => {
                model_path::string_target(service, model_path::list_element(service, target)?)
            }
            Binding::FromAny(alternatives) => {
                if alternatives.is_empty() || alternatives.len() > 8
                    || alternatives.iter().enumerate().any(|(index, alternative)| alternatives.iter().take(index)
                        .any(|prior| prior.operation == alternative.operation)) {
                    return Err(RuntimeCatalogCompileError::Capability);
                }
                Ok(())
            }
            Binding::From { .. } | Binding::Context { .. } | Binding::FromArn { .. } | Binding::FromUrl { .. } | Binding::FromService { .. }
                | Binding::FromBranch { .. } | Binding::ContextFromBranch { .. } => Ok(()),
            Binding::Transformed { binding, transform: crate::overlay::BindingTransform::StripPrefix(prefix) } => {
                if prefix.is_empty() || prefix.len() > 256 || prefix.chars().any(char::is_control)
                    || !matches!(binding.as_ref(), Binding::From { .. } | Binding::Context { .. } | Binding::FromArn { .. } | Binding::FromService { .. }) {
                    return Err(RuntimeCatalogCompileError::Capability);
                }
                model_path::string_target(service, target)
            }
        }
    })
}


fn validate_page(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let input = operation.input()?;
    let output = operation.output()?;
    let token = |pair: &crate::overlay::TokenPair| {
        let (request, response) = (pair.input.to_string(), pair.output.to_string());
        validate_token(service, input, output, &request, &response)
    };
    match overlay.paginator.as_ref() {
        Some(Paginator::Token(pair)) => token(pair),
        Some(Paginator::LastListItem { input: request, output: response, truncated }) => {
            let request = model_path::resolve(service, input, request.as_str())?.1;
            let response = model_path::resolve(service, output, response.as_str())?.1;
            model_path::string_target(service, request)?;
            model_path::string_target(service, model_path::list_element(service, response)?)?;
            let flag = model_path::resolve(service, output, truncated.as_str())?.1;
            if model_path::kind(service, flag)? != "boolean" {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            Ok(())
        }
        Some(Paginator::Tokens { pairs, truncated }) => {
            pairs.iter().try_for_each(token)?;
            truncated.as_ref().map_or(Ok(()), |path| {
                model_path::resolve(service, output, &path.to_string()).map(|_| ())
            })
        }
        Some(Paginator::None) | None => Ok(()),
    }
}

fn validate_token(
    service: Service<'_>,
    input: Target,
    output: Target,
    request: &str,
    response: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    model_path::resolve(service, input, request)?;
    model_path::resolve(service, output, response).map(|_| ())
}

fn validate_purpose_paths(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    if operation.paginator()?.is_some() && overlay.page_size.is_some() {
        let page_size = operation.paginator()?.and_then(|page| page.page_size);
        let path = page_size.ok_or(RuntimeCatalogCompileError::Capability)?;
        model_path::resolve(service, operation.input()?, path)?;
    }
    Ok(())
}
