use super::*;
use crate::overlay::Binding;
use crate::overlay::MemberPath;

pub(super) fn validate(
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
            map_path_error(
                path.as_str(),
                validate_binding(service, input, path, binding),
            )
        })
}

fn validate_binding(
    service: Service<'_>,
    input: Target,
    path: &MemberPath,
    binding: &Binding,
) -> Result<(), RuntimeCatalogCompileError> {
    let target = model_path::resolve(service, input, path.as_str())?.1;
    match binding {
        Binding::Region | Binding::Account => model_path::string_target(service, target),
        Binding::RegionList => validate_region_list(service, target),
        Binding::FromAny(alternatives) => validate_alternatives(alternatives, path),
        Binding::Transformed {
            binding,
            transform: crate::overlay::BindingTransform::StripPrefix(prefix),
        } => validate_transform(service, target, path, binding, prefix),
        Binding::From { .. }
        | Binding::Context { .. }
        | Binding::FromArn { .. }
        | Binding::FromUrl { .. }
        | Binding::FromService { .. }
        | Binding::FromBranch { .. }
        | Binding::ContextFromBranch { .. } => Ok(()),
    }
}

fn validate_region_list(
    service: Service<'_>,
    target: Target,
) -> Result<(), RuntimeCatalogCompileError> {
    model_path::string_target(service, model_path::list_element(service, target)?)
}

fn validate_transform(
    service: Service<'_>,
    target: Target,
    path: &MemberPath,
    binding: &Binding,
    prefix: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    if !super::super::plans::bindings::valid_transform(binding, prefix) {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        });
    }
    model_path::string_target(service, target)
}

fn validate_alternatives(
    alternatives: &[crate::overlay::BindingAlternative],
    path: &MemberPath,
) -> Result<(), RuntimeCatalogCompileError> {
    if alternatives.is_empty()
        || alternatives.len() > 8
        || alternatives.iter().enumerate().any(|(index, alternative)| {
            alternatives
                .iter()
                .take(index)
                .any(|prior| prior.operation == alternative.operation)
        })
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        });
    }
    Ok(())
}
