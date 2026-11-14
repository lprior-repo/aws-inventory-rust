use super::*;

#[tokio::test]
pub(super) async fn failed_scopes_are_durable_idempotent_and_prevent_complete_status() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    store
        .apply_page(
            "one",
            page(
                "ec2",
                Source::Direct,
                vec![resource("i-1", Source::Direct, 10)],
            ),
        )
        .await?;
    let failure = CollectionFailure {
        scope: scope("lambda", Source::Direct),
        category: "access_denied".to_owned(),
        message: "denied '; DROP TABLE scans; --".to_owned(),
    };
    store.record_failure("one", &failure).await?;
    store.record_failure("one", &failure).await?;
    let summary = store.finish_scan("one", ScanStatus::Complete, 20).await?;
    assert_eq!(summary.failed_scopes, 1);
    assert_eq!(summary.completed_scopes, 1);
    assert_eq!(store.scan_status("one").await?, ScanStatus::Partial);
    assert_eq!(
        store
            .query("SELECT category, message FROM collection_failures", 10)
            .await?,
        json!([{"category": "access_denied", "message": failure.message}])
    );
    Ok(())
}
