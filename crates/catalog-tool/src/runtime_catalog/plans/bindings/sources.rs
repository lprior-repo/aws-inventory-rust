use super::*;

type ResourceParts<'a> = (
    i64,
    &'a OperationName,
    &'a MemberPath,
    [Option<(u32, Value)>; 2],
);
type BranchParts<'a> = (&'a OperationName, &'a str, &'a MemberPath);

pub(super) fn encode(
    service: &str,
    target: &MemberPath,
    binding: &Binding,
    services: &super::super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match binding {
        Binding::From { operation, path } => {
            operation_binding(service, target, operation, path, 4, services)
        }
        Binding::FromService {
            service,
            operation,
            path,
        } => operation_binding(service, target, operation, path, 4, services),
        Binding::Context { .. } => context_binding(service, target, binding, services),
        _ => branch_or_resource(service, target, binding, services),
    }
}

fn context_binding(
    service: &str,
    target: &MemberPath,
    binding: &Binding,
    services: &super::super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let Binding::Context {
        operation,
        source,
        path,
    } = binding
    else {
        return Err(path_error(target));
    };
    operation_binding(
        service,
        target,
        operation,
        path,
        selection::source_tag(*source),
        services,
    )
}

fn branch_or_resource(
    service: &str,
    target: &MemberPath,
    binding: &Binding,
    services: &super::super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if let Some((operation, branch, path)) = branch_parts(binding) {
        let source = match binding {
            Binding::ContextFromBranch { source, .. } => *source,
            _ => ContextSource::SelectedItem,
        };
        return branch_binding(service, target, operation, branch, source, path, services);
    }
    resource_binding(service, target, binding, services)
}

fn branch_parts(binding: &Binding) -> Option<BranchParts<'_>> {
    match binding {
        Binding::FromBranch {
            operation,
            branch,
            path,
        }
        | Binding::ContextFromBranch {
            operation,
            branch,
            path,
            ..
        } => Some((operation, branch, path)),
        _ => None,
    }
}

pub(super) fn resource_parts<'a>(
    binding: &'a Binding,
    target: &MemberPath,
) -> Result<ResourceParts<'a>, RuntimeCatalogCompileError> {
    let (tag, operation, path) = match binding {
        Binding::FromArn {
            operation, path, ..
        } => (8, operation, path),
        Binding::FromUrl {
            operation, path, ..
        } => (10, operation, path),
        _ => return Err(path_error(target)),
    };
    let tail = resource_tail(binding).ok_or_else(|| path_error(target))?;
    Ok((tag, operation, path, tail))
}

fn resource_tail(binding: &Binding) -> Option<[Option<(u32, Value)>; 2]> {
    match binding {
        Binding::FromArn {
            arn_service,
            resource_kind,
            ..
        } => Some([
            Some((6, Value::Text(arn_service.clone()))),
            Some((7, Value::Int(selection::arn_kind(*resource_kind)))),
        ]),
        Binding::FromUrl { resource_kind, .. } => {
            Some([Some((6, Value::Int(url_kind(*resource_kind)))), None])
        }
        _ => None,
    }
}

fn url_kind(kind: crate::overlay::UrlResourceKind) -> i64 {
    match kind {
        crate::overlay::UrlResourceKind::SqsQueue => 0,
    }
}
