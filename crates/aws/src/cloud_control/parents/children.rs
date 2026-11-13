//! children operations.
use super::*;

pub(super) async fn collect_child<L, LFut, G, GFut>(
    ctx: &Context,
    child: &ChildType<'_>,
    parent: &ParentContext,
    state: &DiscoveryState,
    list: &L,
    get: &G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    check_cancelled(ctx)?;
    let scope = ctx.scope(
        "cloud_control_list",
        Some(&parent.scope_parent),
        Source::CloudControl,
    );
    if let Err(error) = admit_model(&state.ledger, &parent.scope_parent) {
        state.missing.store(true, Ordering::Relaxed);
        return ctx
            .failure(&scope, error_category(&error), error.to_string())
            .await;
    }
    let identities = Mutex::new(IdentityLedger::default());
    let failure_scope = scope.clone();
    paginate(ctx, scope, |token, budget| {
        let identities = &identities;
        let failure_scope = failure_scope.clone();
        async move {
            check_cancelled(ctx)?;
            let output = match list(ListRequest {
                type_name: child.descriptor.type_name.clone(),
                resource_model: Some(parent.encoded_model.clone()),
                token,
            })
            .await
            {
                Ok(output) => output,
                Err(ApiError::UnsupportedType(message)) => {
                    return record_unsupported(ctx, &failure_scope, message).await;
                }
                Err(error) => return Err(error.into()),
            };
            remember_list_page(child.descriptor, &output, identities, budget)?;
            admit_children(&state.ledger, child.descriptor, &output)?;
            collect_bounded(
                output.resource_descriptions(),
                READ_CONCURRENCY,
                |description| collect_child_item(ctx, child, parent, description, get),
            )
            .await?;
            Ok(PageBody {
                listed_count: output.resource_descriptions().len(),
                next_token: output.next_token().map(str::to_owned),
                ..PageBody::default()
            })
        }
    })
    .await?;
    check_cancelled(ctx)
}

pub(super) async fn collect_child_item<G, GFut>(
    ctx: &Context,
    child: &ChildType<'_>,
    parent: &ParentContext,
    listed: &ResourceDescription,
    get: &G,
) -> Result<(), AwsScanError>
where
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    check_cancelled(ctx)?;
    let id = identifier(listed)?;
    let scope_parent = format!(
        "{}:{:x}",
        parent.scope_parent,
        base16ct::HexDisplay(&Sha256::digest(id.as_bytes()))
    );
    let scope = ctx.scope(
        "cloud_control_get",
        Some(&scope_parent),
        Source::CloudControl,
    );
    let mapped = match get(GetRequest {
        type_name: child.descriptor.type_name.clone(),
        identifier: id.to_owned(),
    })
    .await
    {
        Ok(output) => map_get(ctx, child.descriptor, id, &output)
            .and_then(|body| attach_parent(body, child, parent, true)),
        Err(error) => Err(error.into()),
    };
    match mapped {
        Ok(body) => emit_child(ctx, &scope, body).await,
        Err(error) => {
            emit_failed_read(
                ctx,
                child.descriptor,
                listed,
                &scope,
                error,
                Some((child, parent)),
            )
            .await
        }
    }
}

pub(super) fn attach_parent(
    mut body: PageBody,
    child: &ChildType<'_>,
    parent: &ParentContext,
    complete: bool,
) -> Result<PageBody, AwsScanError> {
    let resource = body
        .resources
        .first_mut()
        .ok_or_else(|| invalid("child mapping has no resource"))?;
    let provider = resource
        .properties
        .get("provider_properties")
        .ok_or_else(|| invalid("child mapping lacks provider properties"))?;
    for (key, expected) in &parent.model {
        match provider.get(key) {
            Some(value) if value.as_str() == Some(expected) => {}
            None if !complete => {}
            _ => {
                return Err(invalid(
                    "child read does not match its requested parent resource model",
                ));
            }
        }
    }
    let properties = resource
        .properties
        .as_object_mut()
        .ok_or_else(|| invalid("child properties are not an object"))?;
    properties.insert("list_resource_model".into(), json!(parent.model));
    properties.insert("parent_resource_id".into(), json!(parent.resource_id));
    properties.insert("parent_type".into(), json!(child.plan.parent_type));
    if complete {
        if body.edges.len() >= MAX_RESOURCE_EDGES {
            return Err(invalid("child relationship cap exceeded"));
        }
        body.edges.push(Edge { from_id: resource.id.clone(), to_id: parent.resource_id.clone(),
            relation: child.plan.relation.clone(), source: resource.source, observed_at: resource.observed_at,
            properties: json!({"target_kind":parent.resource_kind,"cloudformation_type":child.descriptor.type_name,
                "parent_type":child.plan.parent_type,"resource_model":parent.model,
                "evidence":child.plan.evidence,"owner_resolution":"observed_parent_resource_identity"}),
        });
    }
    resource.validate()?;
    Ok(body)
}

pub(super) async fn emit_failed_read(
    ctx: &Context,
    descriptor: &ResourceType,
    listed: &ResourceDescription,
    scope: &CollectionScope,
    error: AwsScanError,
    parent: Option<(&ChildType<'_>, &ParentContext)>,
) -> Result<(), AwsScanError> {
    if matches!(
        error,
        AwsScanError::ChannelClosed | AwsScanError::Api(ApiError::Cancelled)
    ) {
        return Err(error);
    }
    ctx.failure(scope, error_category(&error), error.to_string())
        .await?;
    let fallback =
        map_description(ctx, descriptor, identifier(listed)?, listed, false).and_then(|body| {
            match parent {
                Some((child, context)) => attach_parent(body, child, context, false),
                None => Ok(body),
            }
        });
    match fallback {
        Ok(body) => emit_child(ctx, scope, body).await,
        Err(error) => {
            ctx.failure(
                scope,
                "invalid_response",
                format!("list-only fallback could not be retained: {error}"),
            )
            .await
        }
    }
}
