//! unsupported list and closed consumer never claim empty success behavior.
use super::*;

#[tokio::test]
async fn unsupported_list_and_closed_consumer_never_claim_empty_success() -> TestResult {
    let kind = descriptor("AWS::EC2::Instance")?;
    let (ctx, mut events) = context(3)?;
    collect_type_with(
        &ctx,
        &kind,
        |_| async { Err(ApiError::Permanent("UnsupportedAction".into())) },
        |_| async { Err(ApiError::Permanent("unexpected read".into())) },
    )
    .await?;
    let collected = drain(&mut events)?;
    assert_eq!(collected.len(), 1);
    assert!(matches!(
        collected.first(),
        Some(CollectionEvent::Failure(_))
    ));
    drop(events);
    let result = collect_type_with(
        &ctx,
        &kind,
        |_| async {
            Ok(ListResourcesOutput::builder()
                .resource_descriptions(ResourceDescription::builder().identifier("i-one").build())
                .build())
        },
        |id| async move { Ok(get(description(&id, json!({"InstanceId":id})))) },
    )
    .await;
    assert!(matches!(result, Err(AwsScanError::ChannelClosed)));
    Ok(())
}

#[tokio::test]
async fn pre_cancelled_collection_admits_no_types_or_children() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    ctx.executor.cancel();
    let calls = AtomicUsize::new(0);
    let result = collect_type_with(
        &ctx,
        &kind,
        |_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(ListResourcesOutput::builder().build())
        },
        |_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(ApiError::Permanent("unexpected request".into()))
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert!(matches!(
        collect(&ctx, &[kind], &Selection::Types(Vec::new())).await,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(drain(&mut events)?.is_empty());
    Ok(())
}

#[tokio::test]
async fn mid_page_cancellation_stops_queued_children_without_list_fallbacks() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    let calls = AtomicUsize::new(0);
    let result = collect_type_with(
        &ctx,
        &kind,
        |_| async {
            Ok(ListResourcesOutput::builder()
                .set_resource_descriptions(Some(
                    (0..30)
                        .map(|index| {
                            description(&format!("i-{index}"), json!({"InstanceType":"t3.micro"}))
                        })
                        .collect(),
                ))
                .build())
        },
        |_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            ctx.executor.cancel();
            Err(ApiError::Cancelled)
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert!(calls.load(Ordering::SeqCst) <= READ_CONCURRENCY);
    let collected = drain(&mut events)?;
    assert_eq!(collected.len(), 1);
    assert!(
        matches!(collected.first(), Some(CollectionEvent::Failure(failure)) if failure.category == "cancelled")
    );
    Ok(())
}
