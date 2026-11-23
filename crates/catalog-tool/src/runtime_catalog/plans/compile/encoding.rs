use super::*;
use crate::runtime_catalog::ServiceIndex;

pub(super) fn encode_plan(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    let services = context.services;
    let head = head(prepared, overlay, callable.operation())?;
    let body = overlay_fields(prepared, overlay, services)?;
    let tail = tail(review, callable, context, prepared)?;
    let extensions = extensions(callable, context, prepared)?;
    encode::map(head.into_iter().chain(body).chain(tail).chain(extensions))
}

fn extensions(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl Iterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    let tags = tags::compile(prepared.service, prepared.operation, overlay)?;
    let routing = routing(callable, context, prepared)?;
    let capabilities = capabilities::compile(prepared.service, prepared.operation, overlay)?;
    let discovery = discovery(callable, context)?;
    let map_exclusions = map_exclusions(prepared, overlay)?;
    let row_policy = row_policy(callable, context, prepared)?;
    let emissions = emissions(callable, context, prepared)?;
    Ok(tags
        .map(|value| (14, value))
        .into_iter()
        .chain(routing)
        .chain(capabilities)
        .chain(discovery)
        .chain(map_exclusions)
        .chain(member_exclusions::compile(overlay, context.services)?)
        .chain(row_policy)
        .chain(emissions))
}

fn routing(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl IntoIterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    routing::compile(
        prepared.service,
        prepared.operation,
        callable.plan(),
        context.checked,
        context.services,
    )
}

fn row_policy(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl IntoIterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    row_policy::compile(
        prepared.service,
        prepared.operation,
        &overlay.row_policy,
        overlay.items.as_ref(),
        context.checked,
        callable.service(),
        context.services,
    )
}

fn head(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    operation: &str,
) -> Result<[(u32, Value); 4], RuntimeCatalogCompileError> {
    Ok([
        (1, Value::Text(prepared.service_name.to_owned())),
        (2, Value::Text(operation.to_owned())),
        (3, Value::Text(prepared.kind.to_owned())),
        (4, Value::Int(scope(overlay.scope)?)),
    ])
}

fn tail(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<[(u32, Value); 4], RuntimeCatalogCompileError> {
    let purpose = purpose(callable, context, prepared)?;
    Ok([
        (10, endpoint_value(prepared.endpoint, callable.operation())?),
        (11, review_value(review, callable.origin())?),
        (12, purpose),
        (13, Value::Int(prepared.protocol)),
    ])
}

fn purpose(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    match tag_endpoint::compile(
        prepared.service,
        prepared.operation,
        overlay,
        context.services,
    )? {
        Some(purpose) => Ok(purpose),
        None if overlay.endpoint_discovery => encode::map([(1, Value::Int(4))]),
        None => purpose_value(
            callable.service(),
            callable.operation(),
            context.kinds,
            context.services,
        ),
    }
}

fn discovery(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
) -> Result<[(u32, Value); 2], RuntimeCatalogCompileError> {
    Ok([
        (19, selection::emission(callable.plan())),
        (
            20,
            selection::context_projection(
                callable.service(),
                callable.operation(),
                context.checked,
                context.services,
            )?,
        ),
    ])
}

fn map_exclusions(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if overlay.exclude_map_keys.is_empty() {
        return Ok(None);
    }
    let item = crate::runtime_catalog::selection::target(
        prepared.service,
        prepared.operation.output()?,
        overlay.items.as_ref(),
    )?;
    Ok(Some((
        21,
        super::super::map_exclusions::encode(prepared.service, item, overlay)?,
    )))
}

fn emissions(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if callable.plan().additional_resources.is_empty() {
        return Ok(None);
    }
    let source = emissions::Callable {
        service_name: callable.service(),
        operation_name: callable.operation(),
        primary_kind: prepared.kind,
        service: prepared.service,
        operation: prepared.operation,
    };
    Ok(Some((
        23,
        emissions::compile(source, callable.plan(), context.kinds, context.services)?,
    )))
}

fn overlay_fields(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<[(u32, Value); 5], RuntimeCatalogCompileError> {
    let identity = overlay_identity(prepared, overlay, services)?;
    Ok([
        (5, item_value(overlay.items.as_ref(), services)?),
        (6, identity),
        (
            7,
            binding_values(prepared.service_name, prepared.operation, overlay, services)?,
        ),
        (8, pagination(prepared, overlay, services)?),
        (9, exclusions(overlay, services)?),
    ])
}

fn overlay_identity(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if overlay.emission == crate::overlay::Emission::DiscoveryOnly
        || overlay.tag_endpoint.is_some()
        || overlay.endpoint_discovery
    {
        encode::map([(1, Value::Int(2)), (2, Value::List(Vec::new()))])
    } else {
        identity_value(overlay.identity.as_ref(), prepared.service_name, services)
    }
}

fn pagination(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    compile_page(
        overlay,
        prepared.service,
        prepared.operation,
        prepared.protocol,
        services,
    )
}
