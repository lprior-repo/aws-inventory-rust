//! collection operations.
use super::*;
use aws_sdk_cloudcontrol::{error::SdkError, operation::list_resources::ListResourcesError};

fn list_resources_error(error: SdkError<ListResourcesError>) -> ApiError {
    if let Some(ListResourcesError::TypeNotFoundException(service)) = error.as_service_error() {
        let message = service.message().unwrap_or("no service message");
        return ApiError::UnsupportedType(format!("TypeNotFoundException: {message}"));
    }
    sdk_error(error)
}
#[tracing::instrument(skip_all, fields(resource_types = types.len()))]
pub(crate) async fn collect(
    ctx: &Context,
    types: &[ResourceType],
    selection: &Selection,
) -> Result<(), AwsScanError> {
    if selection.is_universal() {
        return universal::collect(ctx, types).await;
    }
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    if types.len() > TYPE_COUNT {
        return Err(invalid("Cloud Control type bound exceeded"));
    }
    let groups = parents::groups(types)?;
    let client = Client::new(&ctx.config);
    collect_bounded(&groups, TYPE_CONCURRENCY, |group| {
        let client = &client;
        async move {
            if ctx.executor.is_cancelled() {
                return Err(ApiError::Cancelled.into());
            }
            parents::collect_group_with(
                ctx,
                group,
                |request: parents::ListRequest| async move {
                    ctx.execute("cloudcontrol", "read", || async {
                        client
                            .list_resources()
                            .type_name(&request.type_name)
                            .max_results(list_page_size(ctx))
                            .set_resource_model(request.resource_model.clone())
                            .set_next_token(request.token.clone())
                            .send()
                            .await
                            .map_err(list_resources_error)
                    })
                    .await
                },
                |request: parents::GetRequest| async move {
                    ctx.execute("cloudcontrol", "read", || async {
                        client
                            .get_resource()
                            .type_name(&request.type_name)
                            .identifier(&request.identifier)
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await
                },
            )
            .await
        }
    })
    .await?;
    Ok(())
}

pub(super) fn identifier(description: &ResourceDescription) -> Result<&str, AwsScanError> {
    let value = description
        .identifier()
        .ok_or_else(|| invalid("Cloud Control response lacks Identifier"))?;
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.contains('\0') {
        return Err(invalid(
            "Cloud Control Identifier is empty or exceeds its 1024-byte model bound",
        ));
    }
    Ok(value)
}

pub(super) fn validate_response_type(
    expected: &str,
    actual: Option<&str>,
) -> Result<(), AwsScanError> {
    if actual.is_some_and(|actual| actual != expected) {
        return Err(invalid("Cloud Control returned a different resource type"));
    }
    Ok(())
}

pub(super) fn remember_list_page(
    descriptor: &ResourceType,
    output: &ListResourcesOutput,
    ledger: &Mutex<IdentityLedger>,
    budget: usize,
) -> Result<(), AwsScanError> {
    validate_response_type(&descriptor.type_name, output.type_name())?;
    let descriptions = output.resource_descriptions();
    require_budget(descriptions.len(), budget)?;
    if descriptions.len() > MAX_LIST_ITEMS {
        return Err(invalid(
            "Cloud Control service page exceeds the local 1000-item bound",
        ));
    }
    if output
        .next_token()
        .is_some_and(|token| token.is_empty() || token.len() > 4_096)
    {
        return Err(invalid(
            "Cloud Control returned an invalid continuation token",
        ));
    }
    let mut page = BTreeSet::new();
    let mut bytes = 0_usize;
    for description in descriptions {
        let id = identifier(description)?;
        let digest: [u8; 32] = Sha256::digest(id.as_bytes()).into();
        if !page.insert(digest) {
            return Err(invalid("Cloud Control listed the same identifier twice"));
        }
        let properties_bytes = description.properties().map_or(0, str::len);
        bytes = bytes
            .checked_add(properties_bytes)
            .ok_or_else(|| invalid("list property byte overflow"))?;
        if properties_bytes > MAX_PROVIDER_BYTES || bytes > MAX_LIST_BYTES {
            return Err(invalid(
                "Cloud Control list properties exceed the local byte budget",
            ));
        }
    }
    let mut ledger = ledger
        .lock()
        .map_err(|_| invalid("Cloud Control identity ledger was poisoned"))?;
    if ledger.hashes.len().saturating_add(page.len()) > MAX_CHILDREN {
        return Err(AwsScanError::ChildLimit);
    }
    if page.iter().any(|id| ledger.hashes.contains(id)) {
        return Err(invalid("Cloud Control repeated an identifier across pages"));
    }
    ledger.hashes.extend(page);
    Ok(())
}

/// A provider capability boundary is an explicit, accounted outcome rather
/// than an incomplete scope: persist `unsupported_type`, then emit a terminal
/// empty page so the list scope completes and the scan can stay `complete`.
pub(super) async fn record_unsupported(
    ctx: &Context,
    scope: &CollectionScope,
    message: String,
) -> Result<PageBody, AwsScanError> {
    ctx.failure(scope, "unsupported_type", message).await?;
    Ok(PageBody::default())
}

#[tracing::instrument(skip_all, fields(resource_type = descriptor.type_name))]
pub(super) async fn collect_type_with<L, LFut, G, GFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    list: L,
    get: G,
) -> Result<(), AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(String) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let ledger = Mutex::new(IdentityLedger::default());
    let scope = ctx.scope(
        "cloud_control_list",
        Some(&descriptor.type_name),
        Source::CloudControl,
    );
    let failure_scope = scope.clone();
    paginate(ctx, scope, |token, budget| {
        let list = &list;
        let get = &get;
        let ledger = &ledger;
        let failure_scope = failure_scope.clone();
        async move {
            if ctx.executor.is_cancelled() {
                return Err(ApiError::Cancelled.into());
            }
            let output = match list(token).await {
                Ok(output) => output,
                Err(ApiError::UnsupportedType(message)) => {
                    return record_unsupported(ctx, &failure_scope, message).await;
                }
                Err(error) => return Err(error.into()),
            };
            remember_list_page(descriptor, &output, ledger, budget)?;
            collect_bounded(
                output.resource_descriptions(),
                READ_CONCURRENCY,
                |description| async move { enrich_with(ctx, descriptor, description, get).await },
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
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all, fields(resource_type = descriptor.type_name))]
pub(super) async fn enrich_with<G, GFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    listed: &ResourceDescription,
    get: &G,
) -> Result<(), AwsScanError>
where
    G: Fn(String) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let listed_identifier = identifier(listed)?;
    let parent = format!(
        "{}:{:x}",
        descriptor.type_name,
        base16ct::HexDisplay(&Sha256::digest(listed_identifier.as_bytes()))
    );
    let scope = ctx.scope("cloud_control_get", Some(&parent), Source::CloudControl);
    let mapped = match get(listed_identifier.to_owned()).await {
        Ok(output) => map_get(ctx, descriptor, listed_identifier, &output),
        Err(error) => Err(error.into()),
    };
    let outcome = match mapped {
        Ok(body) => emit_child(ctx, &scope, body).await,
        Err(error) => Err(error),
    };
    match outcome {
        Ok(()) => Ok(()),
        Err(AwsScanError::ChannelClosed) => Err(AwsScanError::ChannelClosed),
        Err(AwsScanError::Api(ApiError::Cancelled)) => Err(ApiError::Cancelled.into()),
        Err(error) => {
            // Queue known failure first. Store retains its failed flag when a
            // later thin page is committed in this same child scope.
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            let fallback = match map_description(ctx, descriptor, listed_identifier, listed, false)
            {
                Ok(body) => emit_child(ctx, &scope, body).await,
                Err(error) => Err(error),
            };
            match fallback {
                Ok(()) => {}
                Err(AwsScanError::ChannelClosed) => return Err(AwsScanError::ChannelClosed),
                Err(fallback_error) => {
                    ctx.failure(
                        &scope,
                        "invalid_response",
                        format!("list-only fallback could not be retained: {fallback_error}"),
                    )
                    .await?;
                }
            }
            Ok(())
        }
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn emit_child(
    ctx: &Context,
    scope: &CollectionScope,
    body: PageBody,
) -> Result<(), AwsScanError> {
    let page = CollectionPage {
        scope: scope.clone(),
        page_number: 1,
        next_token: None,
        resources: body.resources,
        edges: body.edges,
    };
    page.validate()?;
    ctx.sender
        .send(CollectionEvent::Page(page))
        .await
        .map_err(|_| AwsScanError::ChannelClosed)?;
    tokio::task::consume_budget().await;
    Ok(())
}
