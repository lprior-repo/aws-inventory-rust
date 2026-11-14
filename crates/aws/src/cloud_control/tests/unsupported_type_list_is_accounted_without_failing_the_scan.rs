use super::*;
use inventory_core::ScanStatus;
use inventory_engine::store::Store;

#[tokio::test]
async fn unsupported_type_list_is_accounted_without_failing_the_scan() -> TestResult {
    let (ctx, mut events) = context(1)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    let outcome = collect_type_with(
        &ctx,
        &kind,
        |_| async {
            Err(ApiError::UnsupportedType(
                "TypeNotFoundException: fixture".into(),
            ))
        },
        |_| async { Err(ApiError::Permanent("unexpected read".into())) },
    )
    .await;
    assert!(outcome.is_ok(), "capability boundary must not abort a scan");

    let events = drain(&mut events)?;
    assert_eq!(events.len(), 2);
    let store = Store::open(":memory:").await?;
    let scan = "scan-explicit";
    store.begin_scan(scan, 1).await?;
    for event in events {
        match event {
            CollectionEvent::Page(page) => {
                assert!(page.resources.is_empty() && page.next_token.is_none());
                store.apply_page(scan, page).await?;
            }
            CollectionEvent::Failure(failure) => {
                assert_eq!(failure.category, "unsupported_type");
                store.record_failure(scan, &failure).await?;
            }
        }
    }
    store.finish_scan(scan, ScanStatus::Complete, 2).await?;
    assert_eq!(store.scan_status(scan).await?, ScanStatus::Complete);
    let summary = store.summary(scan).await?;
    assert_eq!(summary.failed_scopes, 0);
    assert_eq!(summary.completed_scopes, 1);
    Ok(())
}
