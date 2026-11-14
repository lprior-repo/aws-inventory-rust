use super::*;

#[tokio::test]
pub(super) async fn exact_page_replay_is_idempotent_even_after_finalization() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges.push(relationship());
    store.apply_page("one", input.clone()).await?;
    store.apply_page("one", input.clone()).await?;
    let summary = store.finish_scan("one", ScanStatus::Complete, 20).await?;
    assert_eq!(summary.resources, 1);
    assert_eq!(summary.edges, 1);
    assert_eq!(summary.completed_scopes, 1);
    store.apply_page("one", input.clone()).await?;
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM collection_pages", 10)
            .await?,
        json!([{"total": 1}])
    );
    assert_eq!(store.summary("one").await?, summary);
    Ok(())
}
