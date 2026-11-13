//! pagination operations.
use super::*;

/// One SDK page at a time. The only retained history is a bounded set of cursors.
/// Empty pages with continuation tokens are deliberately not terminal.
#[tracing::instrument(skip_all, fields(collector = scope.collector, region = scope.region))]
pub(crate) async fn paginate<F, Fut>(
    ctx: &Context,
    scope: CollectionScope,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn(Option<String>, usize) -> Fut + Send + Sync,
    Fut: Future<Output = Result<PageBody, AwsScanError>> + Send,
{
    // One erased driver per scope bounds trait/layout expansion through nested
    // parent collectors. Requests themselves retain their typed SDK futures.
    let work: BoxFuture<'_, Result<(), AwsScanError>> = Box::pin(paginate_inner(ctx, scope, fetch));
    work.await
}

pub(super) async fn paginate_inner<F, Fut>(
    ctx: &Context,
    scope: CollectionScope,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn(Option<String>, usize) -> Fut + Send + Sync,
    Fut: Future<Output = Result<PageBody, AwsScanError>> + Send,
{
    let pages = stream::unfold(Cursor::default(), |mut cursor| {
        let fetch = &fetch;
        let scope = &scope;
        async move {
            if cursor.done {
                return None;
            }
            if let Some((category, message)) = cursor.pending_failure.take() {
                cursor.done = true;
                let event = CollectionEvent::Failure(CollectionFailure {
                    scope: scope.clone(),
                    category: category.to_owned(),
                    message,
                });
                return Some((Ok(event), cursor));
            }
            match fetch(
                cursor.token.clone(),
                MAX_CHILDREN.saturating_sub(cursor.children),
            )
            .await
            {
                Ok(body) => {
                    if cursor.children.saturating_add(body.listed_count) > MAX_CHILDREN {
                        cursor.done = true;
                        return Some((
                            Ok(CollectionEvent::Failure(CollectionFailure {
                                scope: scope.clone(),
                                category: "child_limit".into(),
                                message: format!(
                                    "scope child cap {MAX_CHILDREN} would be exceeded"
                                ),
                            })),
                            cursor,
                        ));
                    }
                    cursor.advance(&body, ctx.max_pages);
                    let page = CollectionPage {
                        scope: scope.clone(),
                        page_number: cursor.page_number,
                        next_token: body.next_token,
                        resources: body.resources,
                        edges: body.edges,
                    };
                    match page.validate() {
                        Ok(()) => Some((Ok(CollectionEvent::Page(page)), cursor)),
                        Err(error) => {
                            cursor.done = true;
                            Some((
                                Ok(CollectionEvent::Failure(CollectionFailure {
                                    scope: scope.clone(),
                                    category: "invalid_response".into(),
                                    message: error.to_string(),
                                })),
                                cursor,
                            ))
                        }
                    }
                }
                Err(AwsScanError::ChannelClosed) => {
                    cursor.done = true;
                    Some((Err(AwsScanError::ChannelClosed), cursor))
                }
                Err(error) => {
                    cursor.done = true;
                    let event = CollectionEvent::Failure(CollectionFailure {
                        scope: scope.clone(),
                        category: error_category(&error).to_owned(),
                        message: error.to_string(),
                    });
                    Some((Ok(event), cursor))
                }
            }
        }
    });
    pages
        .try_for_each(|event| async move {
            match &event {
                CollectionEvent::Page(page) => {
                    tracing::debug!(collector = %page.scope.collector, page = page.page_number,
                        resources = page.resources.len(), edges = page.edges.len(), "inventory page collected");
                    if page.next_token.is_none() {
                        tracing::info!(collector = %page.scope.collector, pages = page.page_number, "collection scope exhausted its available results");
                    }
                }
                CollectionEvent::Failure(failure) => {
                    tracing::warn!(collector = %failure.scope.collector, category = %failure.category, "collection scope incomplete");
                }
            }
            ctx.sender
                .send(event)
                .await
                .map_err(|_| AwsScanError::ChannelClosed)
        })
        .await
}

pub(crate) async fn collect_bounded<I, F, Fut, T>(
    items: I,
    concurrency: usize,
    fetch: F,
) -> Result<Vec<T>, AwsScanError>
where
    I: IntoIterator + Send,
    I::IntoIter: Send,
    I::Item: Send,
    T: Send,
    F: Fn(I::Item) -> Fut + Send,
    Fut: Future<Output = Result<T, AwsScanError>> + Send,
{
    let work: BoxFuture<'_, Result<Vec<T>, AwsScanError>> = Box::pin(async move {
        stream::iter(items)
            .map(fetch)
            .buffer_unordered(concurrency)
            .try_collect()
            .await
    });
    work.await
}
