use super::super::{encode, selected_path};
use crate::{
    overlay::{
        Binding, CheckedOverlays, ContextSource, Identity, IdentityChoice, IdentityField,
        IdentityLeaf, OperationOverlay, RowPredicate, TagEndpointRequest,
    },
    runtime_catalog::{RuntimeCatalogCompileError, ServiceIndex},
};
use inventory_core::record::Value;
use std::collections::BTreeSet;

#[derive(Default)]
struct Paths {
    selected: BTreeSet<String>,
    response: BTreeSet<String>,
    request: BTreeSet<String>,
}

pub(in crate::runtime_catalog::plans) fn compile(
    service: &str,
    operation: &str,
    checked: &CheckedOverlays,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let mut paths = Paths::default();
    for review in checked.services() {
        for callable in review.callable() {
            overlay_paths(&mut paths, service, operation, review, callable.plan())?;
        }
    }
    encode_paths_map(paths, services)
}

fn overlay_paths(
    paths: &mut Paths,
    service: &str,
    operation: &str,
    review: &crate::overlay::ReviewedService,
    overlay: &crate::overlay::OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    for binding in overlay.bind.values().chain(
        overlay
            .branches
            .iter()
            .flat_map(|branch| branch.bind.values()),
    ) {
        binding_paths(paths, service, operation, review.service(), binding)?;
    }
    if let Some(identity) = &overlay.identity {
        identity_paths(paths, service, operation, review.service(), identity);
    }
    if review.service() == service {
        eligibility_paths(paths, operation, overlay);
        tag_endpoint_paths(paths, operation, overlay);
    }
    Ok(())
}

fn eligibility_paths(
    paths: &mut Paths,
    operation: &str,
    overlay: &crate::overlay::OperationOverlay,
) {
    for eligibility in std::iter::once(&overlay.row_policy)
        .chain(
            overlay
                .branches
                .iter()
                .filter_map(|branch| branch.row_policy.as_ref()),
        )
        .filter_map(|policy| policy.child.as_ref())
        .filter(|child| child.operation.as_str() == operation)
    {
        match &eligibility.predicate {
            RowPredicate::Absent(operand)
            | RowPredicate::BooleanEquals { operand, .. }
            | RowPredicate::StringNotIn { operand, .. }
            | RowPredicate::StringIn { operand, .. } => {
                insert(paths, operand.source, operand.path.as_str())
            }
            RowPredicate::SelectedRequestEquals {
                selected, request, ..
            } => {
                insert(paths, ContextSource::SelectedItem, selected.as_str());
                insert(paths, ContextSource::Request, request.as_str());
            }
        }
    }
}

fn tag_endpoint_paths(
    paths: &mut Paths,
    operation: &str,
    overlay: &crate::overlay::OperationOverlay,
) {
    if let Some(endpoint) = &overlay.tag_endpoint {
        for parent in &endpoint.parents {
            if parent.operation.as_str() != operation {
                continue;
            }
            match &parent.request {
                TagEndpointRequest::Member { parent, .. } => {
                    paths.selected.insert(parent.to_string());
                }
                TagEndpointRequest::Context { parent, .. } => {
                    paths.request.insert(parent.to_string());
                }
                TagEndpointRequest::None | TagEndpointRequest::Owner { .. } => {}
            }
        }
    }
}

fn encode_paths_map(
    paths: Paths,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let encode_paths = |paths: BTreeSet<String>| -> Result<Value, RuntimeCatalogCompileError> {
        if paths.len() > 64 {
            return Err(RuntimeCatalogCompileError::Path {
                path: "selection projection path count".to_owned(),
            });
        }
        Ok(Value::List(
            paths
                .iter()
                .map(|path| selected_path(services, path))
                .collect::<Result<_, _>>()?,
        ))
    };
    encode::map([
        (1, encode_paths(paths.selected)?),
        (2, encode_paths(paths.response)?),
        (3, encode_paths(paths.request)?),
    ])
}

fn binding_paths(
    paths: &mut Paths,
    service: &str,
    operation: &str,
    consumer_service: &str,
    binding: &Binding,
) -> Result<(), RuntimeCatalogCompileError> {
    let inner = binding_inner(binding)?;
    match inner {
        Binding::FromAny(alternatives) if consumer_service == service => {
            from_any_paths(paths, operation, alternatives);
        }
        Binding::From {
            operation: source,
            path,
        }
        | Binding::FromArn {
            operation: source,
            path,
            ..
        }
        | Binding::FromUrl {
            operation: source,
            path,
            ..
        } if consumer_service == service && source.as_str() == operation => {
            paths.selected.insert(path.to_string());
        }
        Binding::FromService {
            service: source_service,
            operation: source,
            path,
        } if source_service == service && source.as_str() == operation => {
            paths.selected.insert(path.to_string());
        }
        other => other_binding_paths(paths, service, operation, consumer_service, other),
    }
    Ok(())
}

fn other_binding_paths(
    paths: &mut Paths,
    service: &str,
    operation: &str,
    consumer_service: &str,
    binding: &Binding,
) {
    match binding {
        Binding::Context {
            operation: source,
            source: namespace,
            path,
        } if consumer_service == service && source.as_str() == operation => {
            insert(paths, *namespace, path.as_str())
        }
        Binding::FromBranch {
            operation: source,
            path,
            ..
        } if consumer_service == service && source.as_str() == operation => {
            paths.selected.insert(path.to_string());
        }
        Binding::ContextFromBranch {
            operation: source,
            source: namespace,
            path,
            ..
        } if consumer_service == service && source.as_str() == operation => {
            insert(paths, *namespace, path.as_str())
        }
        _ => {}
    }
}

fn binding_inner(binding: &Binding) -> Result<&Binding, RuntimeCatalogCompileError> {
    match binding {
        Binding::Transformed { binding, .. } => {
            if matches!(binding.as_ref(), Binding::Transformed { .. }) {
                return Err(RuntimeCatalogCompileError::Path {
                    path: "binding transform nesting".to_owned(),
                });
            }
            Ok(binding.as_ref())
        }
        binding => Ok(binding),
    }
}

fn from_any_paths(
    paths: &mut Paths,
    operation: &str,
    alternatives: &[crate::overlay::BindingAlternative],
) {
    alternatives
        .iter()
        .filter(|alternative| alternative.operation.as_str() == operation)
        .for_each(|alternative| insert(paths, alternative.source, alternative.path.as_str()));
}

fn identity_paths(
    paths: &mut Paths,
    service: &str,
    operation: &str,
    consumer_service: &str,
    identity: &Identity,
) {
    if consumer_service != service {
        return;
    }
    match identity {
        Identity::CorrelatedKey(fields) => fields_paths(paths, operation, fields),
        Identity::Choice(IdentityChoice::PreferArn { fallback, .. }) => {
            leaf_paths(paths, operation, fallback)
        }
        Identity::Choice(IdentityChoice::Discriminated(branches)) => {
            for branch in branches {
                leaf_paths(paths, operation, &branch.identity);
            }
        }
        Identity::Arn(_) | Identity::Template(_) | Identity::Key(_) | Identity::MapArn { .. } => {}
    }
}
fn leaf_paths(paths: &mut Paths, operation: &str, leaf: &IdentityLeaf) {
    if let IdentityLeaf::Fields(fields) = leaf {
        fields_paths(paths, operation, fields)
    }
}
fn fields_paths(paths: &mut Paths, operation: &str, fields: &[IdentityField]) {
    for field in fields {
        if let IdentityField::Context {
            operation: source,
            source: namespace,
            path,
        } = field
            && source.as_str() == operation
        {
            insert(paths, *namespace, path.as_str())
        }
    }
}
fn insert(paths: &mut Paths, source: ContextSource, path: &str) {
    match source {
        ContextSource::SelectedItem => &mut paths.selected,
        ContextSource::Response => &mut paths.response,
        ContextSource::Request => &mut paths.request,
    }
    .insert(path.to_owned());
}

pub(in crate::runtime_catalog::plans) fn emission(overlay: &OperationOverlay) -> Value {
    Value::Int(match overlay.emission {
        crate::overlay::Emission::EmitMetadata => 0,
        crate::overlay::Emission::DiscoveryOnly => 1,
        crate::overlay::Emission::WorkOnly => 2,
    })
}
