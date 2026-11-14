use super::*;

#[tokio::test]
pub(super) async fn universal_unsupported_outcomes_only_count_cloud_control_list_scopes()
-> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("scan", 1).await?;
    store
        .apply_page(
            "scan",
            page("cloud_control_list", Source::CloudControl, Vec::new()),
        )
        .await?;
    store
        .record_failure(
            "scan",
            &CollectionFailure {
                scope: scope("cloud_control_list", Source::CloudControl),
                category: "unsupported_type".to_owned(),
                message: "TypeNotFoundException: fixture".to_owned(),
            },
        )
        .await?;
    store
        .record_failure(
            "scan",
            &CollectionFailure {
                scope: scope("s3", Source::Direct),
                category: "unsupported_type".to_owned(),
                message: "foreign unsupported boundary".to_owned(),
            },
        )
        .await?;
    let outcomes = store.universal_outcomes("scan").await?;
    assert_eq!(outcomes.attempted, 1);
    assert_eq!(outcomes.succeeded, 0);
    assert_eq!(outcomes.unsupported, 1);
    assert_eq!(outcomes.failed, 0);
    assert_eq!(
        outcomes.attempted,
        outcomes
            .succeeded
            .saturating_add(outcomes.unsupported)
            .saturating_add(outcomes.failed)
    );
    Ok(())
}
