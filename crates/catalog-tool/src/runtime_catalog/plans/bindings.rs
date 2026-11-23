use super::*;
use crate::overlay::{BindingTransform, ContextSource, MemberPath, OperationName};

mod sources;

pub(super) fn binding_values(
    service: &str,
    operation: inventory_core::catalog::Operation<'_>,
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let bindings = overlay
        .bind
        .iter()
        .map(|(path, binding)| binding_value(service, path, binding, services))
        .chain(overlay.fixed.iter().map(|(path, value)| {
            let model = catalog_service(services, service)?;
            let target =
                super::super::model_path::resolve(model, operation.input()?, path.as_str())?.1;
            let value = fixed_input::compile(model, target, value, path.as_str())?;
            binding_record(path, 2, (value, Value::Null, Value::Null), services)
        }))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::List(bindings))
}

pub(super) fn binding_value(
    service: &str,
    path: &MemberPath,
    binding: &Binding,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if let Binding::Transformed {
        binding,
        transform: BindingTransform::StripPrefix(prefix),
    } = binding
    {
        if !valid_transform(binding, prefix) {
            return Err(path_error(path));
        }
        return encode::map([
            (1, encode::path(services, path.as_str())?),
            (2, Value::Int(9)),
            (3, plain_binding_value(service, path, binding, services)?),
            (4, Value::Text(prefix.clone())),
        ]);
    }
    plain_binding_value(service, path, binding, services)
}

pub(in crate::runtime_catalog) fn valid_transform(binding: &Binding, prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix.len() <= 256
        && !prefix.chars().any(char::is_control)
        && matches!(
            binding,
            Binding::From { .. }
                | Binding::Context { .. }
                | Binding::FromArn { .. }
                | Binding::FromService { .. }
        )
}

fn plain_binding_value(
    service: &str,
    path: &MemberPath,
    binding: &Binding,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match binding {
        Binding::Account => binding_record(path, 0, nulls(), services),
        Binding::Region => binding_record(path, 1, nulls(), services),
        Binding::RegionList => binding_record(path, 11, nulls(), services),
        Binding::FromAny(alternatives) => {
            binding_alternatives::encode(service, path, alternatives, services)
        }
        _ => sources::encode(service, path, binding, services),
    }
}

fn nulls() -> (Value, Value, Value) {
    (Value::Null, Value::Null, Value::Null)
}
fn path_error(path: &MemberPath) -> RuntimeCatalogCompileError {
    RuntimeCatalogCompileError::Path {
        path: path.as_str().to_owned(),
    }
}

fn operation_binding(
    service: &str,
    target: &MemberPath,
    operation: &OperationName,
    source: &MemberPath,
    tag: i64,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    binding_record(
        target,
        tag,
        (
            Value::Text(service.to_owned()),
            Value::Text(operation.to_string()),
            selected_path(services, source.as_str())?,
        ),
        services,
    )
}

fn resource_binding(
    service: &str,
    target: &MemberPath,
    binding: &Binding,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (tag, operation, path, tail) = sources::resource_parts(binding, target)?;
    encode::map(
        [
            (1, encode::path(services, target.as_str())?),
            (2, Value::Int(tag)),
            (3, Value::Text(service.to_owned())),
            (4, Value::Text(operation.to_string())),
            (5, selected_path(services, path.as_str())?),
        ]
        .into_iter()
        .chain(tail.into_iter().flatten()),
    )
}

pub(super) fn binding_record(
    path: &MemberPath,
    source: i64,
    values: (Value, Value, Value),
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (third, fourth, fifth) = values;
    let present = [(3, third), (4, fourth), (5, fifth)]
        .into_iter()
        .filter(|(_, value)| !matches!(value, Value::Null));
    encode::map(
        [
            (1, encode::path(services, path.as_str())?),
            (2, Value::Int(source)),
        ]
        .into_iter()
        .chain(present),
    )
}

fn branch_binding(
    service: &str,
    target: &MemberPath,
    operation: &OperationName,
    branch: &str,
    source: ContextSource,
    path: &MemberPath,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if !routing::valid_branch_name(branch) {
        return Err(path_error(target));
    }
    encode::map([
        (1, encode::path(services, target.as_str())?),
        (2, Value::Int(13)),
        (3, Value::Text(service.to_owned())),
        (4, Value::Text(operation.to_string())),
        (5, selected_path(services, path.as_str())?),
        (6, Value::Text(branch.to_owned())),
        (7, Value::Int(selection::source_tag(source))),
    ])
}
