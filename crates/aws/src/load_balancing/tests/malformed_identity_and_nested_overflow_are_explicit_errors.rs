//! malformed identity and nested overflow are explicit errors behavior.
use super::*;

#[test]
fn malformed_identity_and_nested_overflow_are_explicit_errors() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    assert!(
        map_load_balancer(
            &ctx,
            &LoadBalancer::builder()
                .r#type(LoadBalancerTypeEnum::Application)
                .build()
        )
        .is_err()
    );
    assert!(map_target_health(&ctx, GROUP, "ip", &registered("not-an-IP", None, None)).is_err());
    assert!(
        map_target_health(&ctx, GROUP, "lambda", &registered("not-an-ARN", None, None)).is_err()
    );
    assert!(
        returned_target_arn(
            "arn:aws:lambda:us-east-1::function:processor",
            "lambda",
            "function:"
        )
        .is_err()
    );
    assert!(map_actions(&[Action::builder().r#type(ActionTypeEnum::Forward).build()]).is_err());
    let actions = vec![weighted_forward(); MAX_NESTED_ITEMS.saturating_add(1)];
    assert!(matches!(
        map_actions(&actions),
        Err(AwsScanError::InvalidData(_))
    ));
    Ok(())
}

#[test]
fn repeated_health_identity_fails_before_storage() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let models = vec![registered("i-123", Some(8080), None); 2];
    let result = map_page(&models, 2, None, |model| {
        map_target_health(&ctx, GROUP, "instance", model)
    });
    assert!(
        matches!(result, Err(AwsScanError::InvalidData(message)) if message.contains("repeats a resource identity"))
    );
    Ok(())
}

#[test]
fn oversized_unpaginated_health_response_is_rejected_before_mapping() {
    let models = vec![(); inventory_core::MAX_PAGE_RESOURCES.saturating_add(1)];
    let calls = std::cell::Cell::new(0usize);
    let result = map_page(&models, crate::common::MAX_CHILDREN, None, |_| {
        calls.set(calls.get().saturating_add(1));
        Ok(PageBody::default())
    });
    assert!(matches!(result, Err(AwsScanError::InvalidData(_))));
    assert_eq!(calls.get(), 0);
}

#[tokio::test]
async fn denied_health_child_does_not_discard_parent_or_claim_empty_success()
-> Result<(), TestError> {
    let (ctx, mut events) = context()?;
    let model = TargetGroup::builder()
        .target_group_arn(GROUP)
        .target_type(TargetTypeEnum::Instance)
        .build();
    paginate(
        &ctx,
        ctx.scope("aws_elbv2_target_group", None, Source::Direct),
        |_token, budget| {
            let model = &model;
            let ctx = &ctx;
            async move {
                let body = map_page(std::slice::from_ref(model), budget, None, |model| {
                    map_target_group(ctx, model)
                })?;
                target_health_with(ctx, GROUP, "instance", || {
                    std::future::ready(Err(ApiError::Denied("fixture AccessDenied".into())))
                })
                .await?;
                Ok(body)
            }
        },
    )
    .await?;
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure)
        if failure.scope.collector == "aws_elbv2_target_health" && failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "access_denied")
    );
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
        if page.scope.collector == "aws_elbv2_target_group" && page.resources.len() == 1 && page.next_token.is_none()));
    assert!(matches!(
        events.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
    Ok(())
}

#[tokio::test]
async fn missing_child_target_is_invalid_response_and_sibling_still_collects()
-> Result<(), TestError> {
    let (ctx, mut events) = context()?;
    let malformed = DescribeTargetHealthOutput::builder()
        .target_health_descriptions(TargetHealthDescription::builder().build())
        .build();
    target_health_with(&ctx, GROUP, "instance", || {
        std::future::ready(Ok(malformed.clone()))
    })
    .await?;
    let valid = DescribeTargetHealthOutput::builder()
        .target_health_descriptions(registered("i-456", Some(80), None))
        .build();
    target_health_with(&ctx, SECOND_GROUP, "instance", || {
        std::future::ready(Ok(valid.clone()))
    })
    .await?;
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure)
        if failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "invalid_response")
    );
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
        if page.scope.parent_id.as_deref() == Some(SECOND_GROUP) && page.resources.len() == 1));
    Ok(())
}

#[tokio::test]
async fn deleted_target_group_is_failure_while_successful_empty_group_has_terminal_page()
-> Result<(), TestError> {
    let (ctx, mut events) = context()?;
    target_health_with(&ctx, GROUP, "instance", || {
        std::future::ready(Err(ApiError::Permanent("TargetGroupNotFound".into())))
    })
    .await?;
    target_health_with(&ctx, SECOND_GROUP, "instance", || {
        std::future::ready(Ok(DescribeTargetHealthOutput::builder().build()))
    })
    .await?;
    assert!(
        matches!(events.try_recv()?, CollectionEvent::Failure(failure)
        if failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "api_error")
    );
    assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
        if page.scope.parent_id.as_deref() == Some(SECOND_GROUP) && page.resources.is_empty() && page.page_number == 1 && page.next_token.is_none()));
    Ok(())
}
