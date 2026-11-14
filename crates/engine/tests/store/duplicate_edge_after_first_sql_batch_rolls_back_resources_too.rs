use super::*;

#[tokio::test]
pub(super) async fn duplicate_edge_after_first_sql_batch_rolls_back_resources_too() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("edge-batch-failure", 1).await?;
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges = (0..128_u32)
        .map(|index| Edge {
            from_id: format!("i-{index}"),
            ..relationship()
        })
        .collect();
    input.edges.push(Edge {
        from_id: "i-0".to_owned(),
        ..relationship()
    });
    let scope = input.scope.clone();
    assert!(store.apply_page("edge-batch-failure", input).await.is_err());
    let summary = store.summary("edge-batch-failure").await?;
    assert_eq!(summary.resources, 0);
    assert_eq!(summary.edges, 0);
    assert!(
        store
            .checkpoint("edge-batch-failure", &scope)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await?,
        json!([{"pages":0}])
    );
    store.close().await;
    Ok(())
}
