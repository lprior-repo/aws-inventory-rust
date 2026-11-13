//! network scope retains nonterminal page then records denial behavior.
use super::*;

#[tokio::test]
async fn network_scope_retains_nonterminal_page_then_records_denial() -> TestResult {
    let (ctx, mut receiver) = context()?;
    let scope = ctx.scope(ROUTE_TABLE.resource_kind, None, Source::Direct);
    paginate(&ctx, scope.clone(), |token, _budget| async move {
        if token.is_none() {
            NetworkPage::default().finish(Some("continue"), 0)
        } else {
            Err(AwsScanError::Api(ApiError::Denied(
                "test access denial".into(),
            )))
        }
    })
    .await?;
    let first = receiver.recv().await.ok_or("missing nonterminal page")?;
    let second = receiver.recv().await.ok_or("missing failure")?;
    assert!(
        matches!(first, CollectionEvent::Page(ref page) if page.scope == scope && page.next_token.as_deref() == Some("continue"))
    );
    assert!(
        matches!(second, CollectionEvent::Failure(ref failure) if failure.scope == scope && failure.category == "access_denied")
    );
    assert!(receiver.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn network_scope_page_cap_emits_failure_without_terminal_success() -> TestResult {
    let (mut ctx, mut receiver) = context()?;
    ctx.max_pages = 1;
    paginate(
        &ctx,
        ctx.scope(ACL.resource_kind, None, Source::Direct),
        |_token, _budget| async { NetworkPage::default().finish(Some("more"), 0) },
    )
    .await?;
    let first = receiver.recv().await.ok_or("missing capped page")?;
    let second = receiver.recv().await.ok_or("missing cap failure")?;
    assert!(
        matches!(first, CollectionEvent::Page(ref page) if page.next_token.as_deref() == Some("more"))
    );
    assert!(
        matches!(second, CollectionEvent::Failure(ref failure) if failure.category == "page_limit")
    );
    assert!(receiver.try_recv().is_err());
    Ok(())
}
