//! driver follows empty page and commits terminal empty page behavior.
use super::*;

#[tokio::test]
async fn driver_follows_empty_page_and_commits_terminal_empty_page()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut events) = context(10)?;
    paginate(
        &ctx,
        ctx.scope("test", None, Source::Direct),
        |token, _budget| async move {
            Ok(PageBody {
                next_token: token.is_none().then(|| "next".to_owned()),
                ..PageBody::default()
            })
        },
    )
    .await?;
    let first = events.try_recv()?;
    let second = events.try_recv()?;
    assert!(
        matches!(first, CollectionEvent::Page(page) if page.page_number == 1 && page.next_token.as_deref() == Some("next"))
    );
    assert!(
        matches!(second, CollectionEvent::Page(page) if page.page_number == 2 && page.next_token.is_none())
    );
    assert!(events.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn driver_records_cap_without_terminal_page() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut events) = context(1)?;
    paginate(
        &ctx,
        ctx.scope("test", None, Source::Direct),
        |_, _budget| async {
            Ok(PageBody {
                next_token: Some("next".into()),
                ..PageBody::default()
            })
        },
    )
    .await?;
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some()));
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "page_limit")
    );
    assert!(events.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn driver_preserves_partial_page_before_permission_failure()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut events) = context(10)?;
    paginate(
        &ctx,
        ctx.scope("test", None, Source::Direct),
        |token, _budget| async move {
            if token.is_some() {
                return Err(ApiError::Denied("fixture denial".into()).into());
            }
            Ok(PageBody {
                next_token: Some("next".into()),
                ..PageBody::default()
            })
        },
    )
    .await?;
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some()));
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "access_denied")
    );
    assert!(events.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn child_budget_is_checked_before_enrichment() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut events) = context(10)?;
    let enriched = Arc::new(AtomicBool::new(false));
    paginate(
        &ctx,
        ctx.scope("test", None, Source::Direct),
        |token, remaining| {
            let enriched = Arc::clone(&enriched);
            async move {
                if token.is_none() {
                    return Ok(PageBody {
                        next_token: Some("last".into()),
                        listed_count: MAX_CHILDREN.saturating_sub(1),
                        ..PageBody::default()
                    });
                }
                require_budget(2, remaining)?;
                enriched.store(true, Ordering::Relaxed);
                Ok(PageBody::default())
            }
        },
    )
    .await?;
    assert!(!enriched.load(Ordering::Relaxed));
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some()));
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "child_limit")
    );
    assert!(events.try_recv().is_err());
    Ok(())
}
