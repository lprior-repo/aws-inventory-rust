use super::*;
use crate::overlay::{CallableOperation, CheckedOverlays};
use std::collections::BTreeMap;
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
        Some(ItemSelector::Member(path) | ItemSelector::Explicit { path, .. }) => path.to_string(),
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
    let kind = operation_kind(context, callable)?;
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
    context: &PlanContext<'p, '_>,
    callable: &CallableOperation,
) -> Result<&'p str, RuntimeCatalogCompileError> {
    let mapped = |operation: &str| context.kinds.operations().iter()
        .find(|row| row.service == callable.service() && row.operation == operation)
        .map(|row| row.kind.as_str());
    if let Some(endpoint) = &callable.plan().tag_endpoint {
        let parent = endpoint.parents.first().ok_or(RuntimeCatalogCompileError::Capability)?;
        return mapped(parent.operation.as_str()).ok_or(RuntimeCatalogCompileError::Capability);
    }
    if callable.plan().endpoint_discovery {
        return context.checked.services().iter().flat_map(|review| review.callable())
            .filter(|candidate| candidate.service() == callable.service()
                && candidate.plan().class == Some(crate::overlay::OverlayClass::Enumerate)
                && candidate.plan().tag_endpoint.is_none() && !candidate.plan().endpoint_discovery)
            .find_map(|candidate| mapped(candidate.operation()))
            .ok_or(RuntimeCatalogCompileError::Capability);
    }
    mapped(callable.operation()).ok_or(RuntimeCatalogCompileError::Capability)
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
    let purpose = match super::tag_endpoint::compile(prepared.service, prepared.operation, overlay, services)? {
        Some(purpose) => purpose,
        None if overlay.endpoint_discovery => encode::map([(1, Value::Int(4))])?,
        None => purpose_value(callable.service(), operation, kinds, services)?,
    };
    let head = [
        (1, Value::Text(prepared.service_name.clone())),
        (2, Value::Text(operation.to_owned())),
        (3, Value::Text(prepared.kind.to_owned())),
        (4, Value::Int(scope(overlay.scope)?)),
    ];
    let tail = [
        (10, endpoint_value(prepared.endpoint, operation)?),
        (11, review_value(review)?),
        (12, purpose),
        (13, Value::Int(prepared.protocol)),
    ];
    let body = overlay_fields(prepared, overlay, context.services)?;
    let tags = super::tags::compile(prepared.service, prepared.operation, overlay)?;
    let routing = super::routing::compile(prepared.service, prepared.operation, overlay, context.checked, services)?;
    let capabilities = super::capabilities::compile(prepared.service, prepared.operation, overlay)?;
    let discovery = [
        (19, super::selection::emission(overlay)),
        (20, super::selection::context_projection(callable.service(), operation, context.checked, services)?),
    ];
    let map_exclusions = if overlay.exclude_map_keys.is_empty() { None } else {
        let item = super::super::selection::target(prepared.service, prepared.operation.output()?, overlay.items.as_ref())?;
        Some((21, super::map_exclusions::encode(prepared.service, item, overlay)?))
    };
    let row_policy = super::row_policy::compile(prepared.service,prepared.operation,&overlay.row_policy,
        overlay.items.as_ref(),context.checked,callable.service(),services)?;
    encode::map(head.into_iter().chain(body).chain(tail).chain(tags.map(|tags| (14, tags)))
        .chain(routing).chain(capabilities).chain(discovery).chain(map_exclusions).chain(row_policy))
}

fn overlay_fields(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &super::super::ServiceIndex<'_>,
) -> Result<[(u32, Value); 5], RuntimeCatalogCompileError> {
    Ok([
        (5, item_value(overlay.items.as_ref(), services)?),
        (6, if overlay.emission == crate::overlay::Emission::DiscoveryOnly || overlay.tag_endpoint.is_some() || overlay.endpoint_discovery {
            encode::map([(1, Value::Int(2)), (2, Value::List(Vec::new()))])?
        } else { identity_value(overlay.identity.as_ref(), &prepared.service_name, services)? }),
        (
            7,
            binding_values(&prepared.service_name, prepared.operation, overlay, services)?,
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
