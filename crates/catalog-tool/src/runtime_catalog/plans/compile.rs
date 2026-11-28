use super::*;
use crate::overlay::{CallableOperation, CheckedOverlays};
use std::collections::BTreeMap;
mod encoding;
use encoding::encode_plan;
struct PlanContext<'p, 'a> {
    checked: &'p CheckedOverlays,
    kinds: &'p KindMap,
    endpoints: &'p BTreeMap<String, VerifiedEndpoint>,
    services: &'p super::super::ServiceIndex<'a>,
}

pub(in crate::runtime_catalog) fn compile<'p, 'a>(
    checked: &'p CheckedOverlays,
    kinds: &'p KindMap,
    endpoints: &'p BTreeMap<String, VerifiedEndpoint>,
    services: &'p super::super::ServiceIndex<'a>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let context = PlanContext {
        checked,
        kinds,
        endpoints,
        services,
    };
    checked
        .services()
        .iter()
        .flat_map(|review| {
            review
                .callable()
                .iter()
                .map(move |callable| (review, callable))
        })
        .map(|(review, callable)| contextual_plan(review, callable, &context))
        .collect()
}
pub(in crate::runtime_catalog) fn decisions(
    checked: &CheckedOverlays,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    checked
        .services()
        .iter()
        .flat_map(|review| {
            review
                .decisions()
                .iter()
                .map(move |decision| (review, decision))
        })
        .map(|(review, decision)| decision_value(services, review, decision))
        .collect()
}

fn decision_value(
    services: &super::super::ServiceIndex<'_>,
    review: &crate::overlay::ReviewedService,
    decision: &crate::overlay::ReviewedDecision,
) -> Result<Value, RuntimeCatalogCompileError> {
    let service = catalog_service(services, review.service())?
        .name()?
        .to_owned();
    let (disposition, reason) = match decision.disposition() {
        crate::overlay::ReviewDisposition::Callable => (0, Value::Null),
        crate::overlay::ReviewDisposition::Denied { reason } => (1, Value::Text(reason.clone())),
        crate::overlay::ReviewDisposition::Deferred { reason } => (2, Value::Text(reason.clone())),
        crate::overlay::ReviewDisposition::Unclassified => (3, Value::Null),
    };
    encode::map([
        (1, Value::Text(service)),
        (2, Value::Text(decision.operation().to_owned())),
        (3, Value::Int(disposition)),
        (4, reason),
    ])
}

fn contextual_plan(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    plan(review, callable, context).map_err(|error| {
        let path = match error {
            RuntimeCatalogCompileError::Capability => context_path(callable.plan()),
            RuntimeCatalogCompileError::Path { path } => path,
            error => return error,
        };
        RuntimeCatalogCompileError::CapabilityContext {
            service: callable.service().to_owned(),
            operation: callable.operation().to_owned(),
            path,
        }
    })
}

fn context_path(overlay: &OperationOverlay) -> String {
    match overlay.items.as_ref() {
        Some(ItemSelector::Member(path) | ItemSelector::Explicit { path, .. }) => path.to_string(),
        Some(ItemSelector::Nested(nested)) => nested.parents.to_string(),
        Some(ItemSelector::Root) => "response".to_owned(),
        None => "operation metadata".to_owned(),
    }
}

struct PreparedPlan<'p, 'a> {
    kind: &'p str,
    service_name: &'a str,
    service: inventory_core::catalog::Service<'a>,
    operation: inventory_core::catalog::Operation<'a>,
    endpoint: &'p VerifiedEndpoint,
    protocol: i64,
}

fn plan(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let prepared = prepare_plan(callable, context)?;
    encode_plan(review, callable, context, &prepared)
}

fn prepare_plan<'p, 'a>(
    callable: &CallableOperation,
    context: &PlanContext<'p, 'a>,
) -> Result<PreparedPlan<'p, 'a>, RuntimeCatalogCompileError> {
    let kind = operation_kind(context, callable)?;
    let service = catalog_service(context.services, callable.service())?;
    let operation = catalog_operation(service, callable.operation())?;
    plan_validation::validate(service, operation, callable.plan())?;
    let endpoint = context.endpoints.get(callable.service());
    Ok(PreparedPlan {
        kind,
        service_name: service.name()?,
        service,
        operation,
        endpoint: endpoint.ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: callable.service().to_owned(),
        })?,
        protocol: protocol(service)?,
    })
}

fn operation_kind<'p>(
    context: &PlanContext<'p, '_>,
    callable: &CallableOperation,
) -> Result<&'p str, RuntimeCatalogCompileError> {
    let mapped = |operation: &str| {
        context
            .kinds
            .operations()
            .iter()
            .find(|row| row.service == callable.service() && row.operation == operation)
            .map(|row| row.kind.as_str())
    };
    if let Some(endpoint) = &callable.plan().tag_endpoint {
        let parent = endpoint
            .parents
            .first()
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: callable.service().to_owned(),
            })?;
        return mapped(parent.operation.as_str()).ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: parent.operation.as_str().to_owned(),
        });
    }
    if callable.plan().endpoint_discovery {
        return context
            .checked
            .services()
            .iter()
            .flat_map(|review| review.callable())
            .filter(|candidate| {
                candidate.service() == callable.service()
                    && candidate.plan().class == Some(crate::overlay::OverlayClass::Enumerate)
                    && candidate.plan().tag_endpoint.is_none()
                    && !candidate.plan().endpoint_discovery
            })
            .find_map(|candidate| mapped(candidate.operation()))
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: callable.service().to_owned(),
            });
    }
    mapped(callable.operation()).ok_or_else(|| RuntimeCatalogCompileError::Path {
        path: callable.operation().to_owned(),
    })
}
