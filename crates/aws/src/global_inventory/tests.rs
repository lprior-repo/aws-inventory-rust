use super::*;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use serde_json::json;
use std::sync::Arc;

pub(super) fn context()
-> Result<(Context, tokio::sync::mpsc::Receiver<CollectionEvent>), Box<dyn std::error::Error>> {
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    let config = aws_types::SdkConfig::builder()
        .region(Region::new("eu-west-1"))
        .build();
    Ok((
        Context {
            config,
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "123456789012".into(),
            partition: "aws".into(),
            region: "global".into(),
            max_pages: 10,
            page_size: 100,
        },
        receiver,
    ))
}

#[test]
fn global_scope_does_not_become_sdk_endpoint_region() -> Result<(), Box<dyn std::error::Error>> {
    let (base, _rx) = context()?;
    let ctx = global_context(&base)?;
    assert_eq!(ctx.region, "global");
    assert_eq!(ctx.config.region().map(Region::as_ref), Some("us-east-1"));
    assert_eq!(endpoint_region("aws-cn")?, "cn-north-1");
    assert_eq!(endpoint_region("aws-us-gov")?, "us-gov-west-1");
    assert!(endpoint_region("aws-iso").is_err());
    Ok(())
}

#[test]
fn marker_requires_evidence_of_more_pages() -> Result<(), AwsScanError> {
    assert_eq!(marker(false, Some("stale"))?, None);
    assert_eq!(marker(true, Some("next"))?, Some("next".into()));
    assert!(marker(true, None).is_err());
    assert!(marker(true, Some("")).is_err());
    Ok(())
}

#[test]
fn identity_ledger_rejects_duplicates_and_budget_before_commit() -> Result<(), AwsScanError> {
    let ledger = SeenIds::default();
    assert!(ledger.record(["a", "b"].into_iter(), 1).is_err());
    assert_eq!(ledger.record(["a", "b"].into_iter(), 2)?, 2);
    assert!(ledger.record(["b"].into_iter(), 2).is_err());
    assert!(ledger.record(["c", "c"].into_iter(), 2).is_err());
    assert_eq!(ledger.record(["c"].into_iter(), 1)?, 1);
    Ok(())
}

#[test]
fn compound_identity_is_unambiguous() -> Result<(), AwsScanError> {
    assert_ne!(
        digest_identity("record", &["ab", "c"])?,
        digest_identity("record", &["a", "bc"])?
    );
    assert_eq!(dns_reference("EXAMPLE.com.")?, "dns:example.com");
    assert!(dns_reference("https://credential@example.com").is_err());
    assert!(dns_reference("credential example.com").is_err());
    assert!(digest_identity("record", &["value"; 17]).is_err());
    Ok(())
}

#[tokio::test]
async fn detail_failure_precedes_fallback_and_keeps_scope_incomplete()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    let scope = ctx.scope("aws_iam_role", Some("role-id"), Source::Direct);
    let resource = ctx.resource(
        "aws_iam_role",
        "role-id".into(),
        None,
        json!({"details_available":false}),
        BTreeMap::new(),
    )?;
    observed(
        &ctx,
        scope.clone(),
        Err(ApiError::Denied("denied".into()).into()),
        Some(single(resource, Vec::new())),
    )
    .await?;
    assert!(matches!(rx.try_recv()?, CollectionEvent::Failure(failure)
        if failure.scope == scope && failure.category == "access_denied"));
    assert!(matches!(rx.try_recv()?, CollectionEvent::Page(page)
        if page.scope == scope && page.resources.len() == 1));
    assert!(rx.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn invalid_child_metadata_does_not_drop_valid_siblings()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    crate::common::collect_bounded(["bad", "good"], 2, |id| {
        let ctx = &ctx;
        async move {
            let resource = ctx.resource(
                "synthetic_global",
                if id == "bad" {
                    String::new()
                } else {
                    id.into()
                },
                None,
                json!({}),
                BTreeMap::new(),
            )?;
            observed(
                ctx,
                ctx.scope("synthetic_global", Some(id), Source::Direct),
                Ok(single(resource, Vec::new())),
                None,
            )
            .await
        }
    })
    .await?;
    let mut failures = 0usize;
    let mut valid = 0usize;
    while let Ok(event) = rx.try_recv() {
        match event {
            CollectionEvent::Failure(failure) => {
                assert_eq!(failure.scope.parent_id.as_deref(), Some("bad"));
                failures = failures.saturating_add(1);
            }
            CollectionEvent::Page(page) => {
                assert_eq!(
                    page.resources.first().map(|resource| resource.id.as_str()),
                    Some("good")
                );
                valid = valid.saturating_add(1);
            }
        }
    }
    assert_eq!((failures, valid), (1, 1));
    Ok(())
}

#[tokio::test]
async fn cancellation_and_closed_consumers_never_become_fallback_success()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    ctx.executor.cancel();
    let scope = ctx.scope("synthetic_global", Some("cancelled"), Source::Direct);
    let result = observed(
        &ctx,
        scope,
        Err(ApiError::Cancelled.into()),
        Some(PageBody::default()),
    )
    .await;
    assert!(matches!(
        result,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert!(
        matches!(rx.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "cancelled")
    );
    assert!(rx.try_recv().is_err());

    let (active, receiver) = context()?;
    drop(receiver);
    assert!(matches!(
        observed(
            &active,
            active.scope("synthetic_global", None, Source::Direct),
            Ok(PageBody::default()),
            None
        )
        .await,
        Err(AwsScanError::ChannelClosed)
    ));
    Ok(())
}

#[tokio::test]
async fn unavailable_section_is_distinct_from_successful_absence()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    let (_, denied) = section(&ctx, "bucket", "policy_status", || async {
        Err(ApiError::Denied("denied".into()).into())
    })
    .await?;
    let (_, absent) = section(&ctx, "bucket", "policy_status", || async {
        Ok(json!({"configured":false}))
    })
    .await?;
    assert_eq!(denied, None);
    assert_eq!(absent, Some(json!({"configured":false})));
    assert!(matches!(rx.try_recv()?, CollectionEvent::Failure(_)));
    assert!(matches!(rx.try_recv()?, CollectionEvent::Page(_)));
    Ok(())
}

#[tokio::test]
async fn pre_cancelled_global_scan_never_constructs_service_work()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    ctx.executor.cancel();
    let services = GLOBAL_SERVICES.into_iter().map(str::to_owned).collect();
    assert!(matches!(
        collect(&ctx, &services).await,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(ctx.executor.stats().attempts, 0);
    assert!(rx.try_recv().is_err());
    Ok(())
}
