//! discovery operations.
use super::*;

pub(in super::super) async fn collect_group_with<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
    list: L,
    get: G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if group.children.is_empty() {
        return collect_type_with(
            ctx,
            group.descriptor,
            |token| {
                list(ListRequest {
                    type_name: group.descriptor.type_name.clone(),
                    resource_model: None,
                    token,
                })
            },
            |identifier| {
                get(GetRequest {
                    type_name: group.descriptor.type_name.clone(),
                    identifier,
                })
            },
        )
        .await;
    }
    let state = DiscoveryState::default();
    collect_parent_pages(ctx, group, &state, &list, &get).await?;
    finish_discovery(ctx, group, &state).await
}

pub(in super::super) async fn collect_parent_pages<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
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
    let identities = Mutex::new(IdentityLedger::default());
    paginate(
        ctx,
        ctx.scope(
            "cloud_control_list",
            Some(&group.descriptor.type_name),
            Source::CloudControl,
        ),
        |token, budget| {
            let identities = &identities;
            async move {
                check_cancelled(ctx)?;
                let output = list(ListRequest {
                    type_name: group.descriptor.type_name.clone(),
                    resource_model: None,
                    token,
                })
                .await?;
                remember_list_page(group.descriptor, &output, identities, budget)?;
                collect_bounded(
                    output.resource_descriptions(),
                    READ_CONCURRENCY,
                    |description| collect_parent_item(ctx, group, state, description, list, get),
                )
                .await?;
                if output.next_token().is_none() {
                    state.exhausted.store(true, Ordering::Relaxed);
                }
                Ok(PageBody {
                    listed_count: output.resource_descriptions().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

pub(in super::super) fn check_cancelled(ctx: &Context) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

pub(in super::super) async fn collect_parent_item<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
    state: &DiscoveryState,
    listed: &ResourceDescription,
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
    let id = identifier(listed)?;
    let parent_scope = format!(
        "{}:{:x}",
        group.descriptor.type_name,
        base16ct::HexDisplay(&Sha256::digest(id.as_bytes()))
    );
    let scope = ctx.scope(
        "cloud_control_get",
        Some(&parent_scope),
        Source::CloudControl,
    );
    let mapped = match get(GetRequest {
        type_name: group.descriptor.type_name.clone(),
        identifier: id.to_owned(),
    })
    .await
    {
        Ok(output) => map_get(ctx, group.descriptor, id, &output),
        Err(error) => Err(error.into()),
    };
    match mapped {
        Ok(body) => {
            let resource = body
                .resources
                .first()
                .ok_or_else(|| invalid("mapped parent has no resource"))?;
            let contexts: Vec<_> = group
                .children
                .iter()
                .map(|child| (child, parent_context(child, resource)))
                .collect();
            emit_child(ctx, &scope, body).await?;
            collect_bounded(
                contexts,
                CHILD_TYPE_CONCURRENCY,
                |(child, context)| async move {
                    match context {
                        Ok(context) => collect_child(ctx, child, &context, state, list, get).await,
                        Err(error) => {
                            state.missing.store(true, Ordering::Relaxed);
                            let failure_scope = ctx.scope(
                                "cloud_control_parent_context",
                                Some(&format!(
                                    "{}:{:x}",
                                    child.descriptor.type_name,
                                    base16ct::HexDisplay(&Sha256::digest(id.as_bytes()))
                                )),
                                Source::CloudControl,
                            );
                            ctx.failure(
                                &failure_scope,
                                "parent_context_unavailable",
                                error.to_string(),
                            )
                            .await
                        }
                    }
                },
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            state.missing.store(true, Ordering::Relaxed);
            emit_failed_read(ctx, group.descriptor, listed, &scope, error, None).await
        }
    }
}

pub(in super::super) async fn finish_discovery(
    ctx: &Context,
    group: &RootGroup<'_>,
    state: &DiscoveryState,
) -> Result<(), AwsScanError> {
    check_cancelled(ctx)?;
    let complete =
        state.exhausted.load(Ordering::Relaxed) && !state.missing.load(Ordering::Relaxed);
    for child in &group.children {
        let scope = ctx.scope(
            "cloud_control_parent_discovery",
            Some(&child.descriptor.type_name),
            Source::CloudControl,
        );
        if complete {
            emit_child(ctx, &scope, PageBody::default()).await?;
        } else {
            ctx.failure(&scope, "parent_discovery_incomplete",
                "parent enumeration or a required parent read did not complete; child inventory may be incomplete").await?;
        }
    }
    Ok(())
}
