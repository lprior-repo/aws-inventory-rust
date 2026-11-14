use super::*;

#[tracing::instrument(skip_all, fields(collector = %scope.collector))]
pub(super) async fn emit(
    ctx: &Context,
    scope: CollectionScope,
    body: PageBody,
) -> Result<(), AwsScanError> {
    let page = checked_page(scope, body)?;
    ctx.sender
        .send(CollectionEvent::Page(page))
        .await
        .map_err(|_| AwsScanError::ChannelClosed)
}

pub(super) fn checked_page(
    scope: CollectionScope,
    body: PageBody,
) -> Result<CollectionPage, AwsScanError> {
    let page = CollectionPage {
        scope,
        page_number: 1,
        next_token: body.next_token,
        resources: body.resources,
        edges: body.edges,
    };
    page.validate()?;
    Ok(page)
}

#[tracing::instrument(skip_all, fields(collector = %scope.collector))]
pub(super) async fn observed(
    ctx: &Context,
    scope: CollectionScope,
    result: Result<PageBody, AwsScanError>,
    fallback: Option<PageBody>,
) -> Result<(), AwsScanError> {
    match result.and_then(|body| checked_page(scope.clone(), body)) {
        Ok(page) => {
            ensure_active(ctx)?;
            ctx.sender
                .send(CollectionEvent::Page(page))
                .await
                .map_err(|_| AwsScanError::ChannelClosed)
        }
        Err(AwsScanError::ChannelClosed) => Err(AwsScanError::ChannelClosed),
        Err(error) => {
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            if matches!(error, AwsScanError::Api(ApiError::Cancelled)) {
                return Err(error);
            }
            ensure_active(ctx)?;
            if let Some(body) = fallback {
                emit(ctx, scope, body).await?;
            }
            Ok(())
        }
    }
}

#[tracing::instrument(skip_all, fields(section))]
pub(super) async fn section<F, Fut>(
    ctx: &Context,
    parent: &str,
    section: &'static str,
    fetch: F,
) -> Result<(&'static str, Option<Value>), AwsScanError>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = Result<Value, AwsScanError>> + Send,
{
    ensure_active(ctx)?;
    let scope = ctx.scope(section, Some(parent), Source::Direct);
    match fetch().await {
        Ok(value) => {
            ensure_active(ctx)?;
            emit(ctx, scope, PageBody::default()).await?;
            Ok((section, Some(value)))
        }
        Err(error) => {
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            if matches!(
                error,
                AwsScanError::Api(ApiError::Cancelled) | AwsScanError::ChannelClosed
            ) {
                return Err(error);
            }
            Ok((section, None))
        }
    }
}

pub(super) fn single(resource: Resource, edges: Vec<Edge>) -> PageBody {
    PageBody {
        resources: vec![resource],
        edges,
        listed_count: 1,
        ..PageBody::default()
    }
}

pub(super) fn edge_set(edges: impl IntoIterator<Item = Edge>) -> Result<Vec<Edge>, AwsScanError> {
    let mut merged = BTreeMap::new();
    for (index, edge) in edges
        .into_iter()
        .take(MAX_COMPONENTS.saturating_add(1))
        .enumerate()
    {
        if index == MAX_COMPONENTS {
            return Err(AwsScanError::ChildLimit);
        }
        let evidence = edge.properties.clone();
        let retained: serde_json::Map<String, Value> = [
            "owner_resolution",
            "target_owner_resolution",
            "target_resource_id",
            "target_kind",
            "target_region",
        ]
        .into_iter()
        .filter_map(|key| {
            evidence
                .get(key)
                .map(|value| (key.to_owned(), value.clone()))
        })
        .collect();
        crate::ecs::merge_reference_edge(
            &mut merged,
            Edge {
                properties: Value::Object(retained),
                ..edge
            },
            evidence,
        )?;
    }
    Ok(merged.into_values().collect())
}
