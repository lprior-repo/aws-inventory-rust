use super::*;

#[tokio::test]
pub(super) async fn partial_snapshot_never_erases_previous_resources_or_edges() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("old", 1).await?;
    let mut old = page(
        "ec2",
        Source::Direct,
        vec![
            resource("i-1", Source::Direct, 10),
            resource("i-2", Source::Direct, 10),
        ],
    );
    old.edges.push(relationship());
    store.apply_page("old", old.clone()).await?;
    store.finish_scan("old", ScanStatus::Complete, 20).await?;
    store.begin_scan("new", 30).await?;
    store
        .apply_page(
            "new",
            page(
                "ec2",
                Source::Direct,
                vec![resource("i-1", Source::Direct, 40)],
            ),
        )
        .await?;
    store
        .record_failure(
            "new",
            &CollectionFailure {
                scope: scope("lambda", Source::Direct),
                category: "timeout".to_owned(),
                message: "late".to_owned(),
            },
        )
        .await?;
    store.finish_scan("new", ScanStatus::Complete, 50).await?;
    assert_eq!(store.scan_status("new").await?, ScanStatus::Partial);
    assert_eq!(store.resources("new").await?.len(), 1);
    assert_eq!(
        store
            .query("SELECT id FROM latest_resources ORDER BY id", 10)
            .await?,
        json!([{"id":"i-1"},{"id":"i-2"}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_edges", 10)
            .await?,
        json!([{"total":1}])
    );
    Ok(())
}
