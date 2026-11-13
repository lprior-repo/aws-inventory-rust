//! empty list page with token continues to a revision and terminal page behavior.
use super::*;

#[tokio::test]
async fn empty_list_page_with_token_continues_to_a_revision_and_terminal_page() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    collect_with(
        &ctx,
        |token| async move {
            Ok(if token.is_none() {
                ListTaskDefinitionsOutput::builder()
                    .next_token("next")
                    .build()
            } else {
                ListTaskDefinitionsOutput::builder()
                    .task_definition_arns(ARN)
                    .build()
            })
        },
        |arn| async move { Ok(output(definition(&arn, 7))) },
    )
    .await?;
    let events = drain(&mut receiver)?;
    assert_eq!(events.len(), 3);
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.page_number == 1 && page.resources.is_empty() && page.next_token.as_deref() == Some("next"))));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.page_number == 2 && page.next_token.is_none())));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == DEFINITION_KIND && page.scope.parent_id.as_deref() == Some(ARN) && page.resources.len() == 1)));
    Ok(())
}

#[tokio::test]
async fn denied_description_is_separate_from_successful_sibling_and_list_coverage() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    collect_with(
        &ctx,
        |_| async {
            Ok(ListTaskDefinitionsOutput::builder()
                .task_definition_arns(ARN)
                .task_definition_arns(SECOND)
                .build())
        },
        |arn| async move {
            if arn == ARN {
                Err(ApiError::Denied("fixture denied".into()))
            } else {
                Ok(output(definition(&arn, 8)))
            }
        },
    )
    .await?;
    let events = drain(&mut receiver)?;
    assert_eq!(events.len(), 3);
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == DEFINITION_KIND && failure.scope.parent_id.as_deref() == Some(ARN) && failure.category == "access_denied")));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(SECOND) && page.resources.len() == 1)));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.is_none())));
    assert!(!events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(ARN))));
    Ok(())
}

#[tokio::test]
async fn malformed_description_is_contained_to_its_revision_scope() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    collect_with(
        &ctx,
        |_| async {
            Ok(ListTaskDefinitionsOutput::builder()
                .task_definition_arns(ARN)
                .task_definition_arns(SECOND)
                .build())
        },
        |arn| async move { Ok(output(definition(&arn, 8))) },
    )
    .await?;
    let events = drain(&mut receiver)?;
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.parent_id.as_deref() == Some(ARN) && failure.category == "invalid_response")));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(SECOND) && page.resources.len() == 1)));
    Ok(())
}

#[tokio::test]
async fn denied_list_does_not_describe_or_claim_empty_success() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let calls = AtomicUsize::new(0);
    collect_with(
        &ctx,
        |_| async { Err(ApiError::Denied("fixture list denied".into())) },
        |arn| {
            calls.fetch_add(1, Ordering::Relaxed);
            async move { Ok(output(definition(&arn, 7))) }
        },
    )
    .await?;
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let events = drain(&mut receiver)?;
    assert_eq!(events.len(), 1);
    assert!(
        matches!(events.first(), Some(CollectionEvent::Failure(failure)) if failure.scope.collector == LIST_SCOPE && failure.category == "access_denied")
    );
    Ok(())
}

#[tokio::test]
async fn page_cap_retains_token_and_explicit_incomplete_scope() -> TestResult {
    let (ctx, mut receiver) = context(1)?;
    collect_with(
        &ctx,
        |_| async {
            Ok(ListTaskDefinitionsOutput::builder()
                .task_definition_arns(ARN)
                .next_token("more")
                .build())
        },
        |arn| async move { Ok(output(definition(&arn, 7))) },
    )
    .await?;
    let events = drain(&mut receiver)?;
    assert_eq!(events.len(), 3);
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == LIST_SCOPE && failure.category == "page_limit")));
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.as_deref() == Some("more"))));
    assert!(!events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.is_none())));
    Ok(())
}

#[tokio::test]
async fn repeated_revision_across_pages_does_not_replay_description_scope() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let calls = AtomicUsize::new(0);
    collect_with(
        &ctx,
        |token| async move {
            Ok(ListTaskDefinitionsOutput::builder()
                .task_definition_arns(ARN)
                .set_next_token(token.is_none().then(|| "next".into()))
                .build())
        },
        |arn| {
            calls.fetch_add(1, Ordering::Relaxed);
            async move { Ok(output(definition(&arn, 7))) }
        },
    )
    .await?;
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let events = drain(&mut receiver)?;
    assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == LIST_SCOPE && failure.category == "invalid_response")));
    assert_eq!(events.iter().filter(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == DEFINITION_KIND)).count(), 1);
    Ok(())
}

#[tokio::test]
async fn malformed_continuation_is_not_silently_declared_complete() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    collect_with(
        &ctx,
        |_| async { Ok(ListTaskDefinitionsOutput::builder().next_token("").build()) },
        |arn| async move { Ok(output(definition(&arn, 7))) },
    )
    .await?;
    let events = drain(&mut receiver)?;
    assert_eq!(events.len(), 1);
    assert!(
        matches!(events.first(), Some(CollectionEvent::Failure(failure)) if failure.category == "invalid_response")
    );
    Ok(())
}
