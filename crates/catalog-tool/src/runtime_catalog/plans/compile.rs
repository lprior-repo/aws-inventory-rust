use super::*;
use crate::overlay::{CallableOperation, CheckedOverlays};
use std::collections::BTreeMap;
struct PlanContext<'p, 'a> {
    kinds: &'p KindMap,
    endpoints: &'p BTreeMap<String, VerifiedEndpoint>,
    services: &'p super::super::ServiceIndex<'a>,
}

pub(in crate::runtime_catalog) fn compile<'p, 'a>(
    checked: &CheckedOverlays,
    kinds: &'p KindMap,
    endpoints: &'p BTreeMap<String, VerifiedEndpoint>,
    services: &'p super::super::ServiceIndex<'a>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let context = PlanContext {
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
    let service = callable.service().to_owned();
    let operation = callable.operation().to_owned();
    let path = context_path(callable.plan());
    plan(review, callable, context).map_err(|error| match error {
        RuntimeCatalogCompileError::Capability => RuntimeCatalogCompileError::CapabilityContext {
            service,
            operation,
            path,
        },
        error => error,
    })
}

fn context_path(overlay: &OperationOverlay) -> String {
    match overlay.items.as_ref() {
        Some(ItemSelector::Member(path)) => path.to_string(),
        Some(ItemSelector::Root) => "response".to_owned(),
        None => "operation metadata".to_owned(),
    }
}

struct PreparedPlan<'p, 'a> {
    kind: &'p str,
    service_name: String,
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
    let kind = operation_kind(context.kinds, callable)?;
    let service = catalog_service(context.services, callable.service())?;
    let operation = catalog_operation(service, callable.operation())?;
    plan_validation::validate(service, operation, callable.plan())?;
    let endpoint = context.endpoints.get(callable.service());
    Ok(PreparedPlan {
        kind,
        service_name: service.name()?.to_owned(),
        service,
        operation,
        endpoint: endpoint.ok_or(RuntimeCatalogCompileError::Capability)?,
        protocol: protocol(service)?,
    })
}

fn operation_kind<'p>(
    kinds: &'p KindMap,
    callable: &CallableOperation,
) -> Result<&'p str, RuntimeCatalogCompileError> {
    kinds
        .operations()
        .iter()
        .find(|row| row.service == callable.service() && row.operation == callable.operation())
        .map(|row| row.kind.as_str())
        .ok_or(RuntimeCatalogCompileError::Capability)
}

fn encode_plan(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    let operation = callable.operation();
    let (kinds, services) = (context.kinds, context.services);
    let purpose = purpose_value(callable.service(), operation, kinds, services)?;
    let head = [
        (1, Value::Text(prepared.service_name.clone())),
        (2, Value::Text(operation.to_owned())),
        (3, Value::Text(prepared.kind.to_owned())),
        (4, Value::Int(scope(overlay.scope)?)),
    ];
    let tail = [
        (10, endpoint_value(prepared.endpoint)?),
        (11, review_value(review)?),
        (12, purpose),
        (13, Value::Int(prepared.protocol)),
    ];
    let body = overlay_fields(prepared, overlay, context.services)?;
    encode::map(head.into_iter().chain(body).chain(tail))
}

fn overlay_fields(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<[(u32, Value); 5], RuntimeCatalogCompileError> {
    Ok([
        (5, item_value(overlay.items.as_ref(), services)?),
        (6, identity_value(overlay.identity.as_ref(), services)?),
        (
            7,
            binding_values(&prepared.service_name, overlay, services)?,
        ),
        (8, plan_page(prepared, overlay, services)?),
        (9, exclusions(overlay, services)?),
    ])
}

fn plan_page(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    compile_page(
        overlay,
        prepared.service,
        prepared.operation,
        prepared.protocol,
        services,
    )
}
