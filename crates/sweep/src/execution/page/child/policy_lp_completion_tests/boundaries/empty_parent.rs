use super::*;
#[tokio::test]
async fn given_a_native_graph_with_empty_request_parent_field_when_child_work_is_attempted_then_no_phantom_endpoint_can_persist()
-> TestResult {
    // The parent ARN is valid; its separate queryable id is exactly empty text. A required child
    // request source that is empty is absent: the child is skipped for that row, so no request
    // with an empty correlated operand is ever scheduled.
    let bytes = compiled(
        "neptune-graph",
        "2023-11-29",
        &["ListGraphs", "ListPrivateGraphEndpoints"],
        "AWS::NeptuneGraph::Graph",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "neptune-empty-request-parent").await?;
    let root = plan(&catalog, "neptune-graph", "ListGraphs")?;
    scenario.seed(root).await?;
    let lease = scenario.claim().await?;
    let (request, _) = scenario.request(root, &lease)?;
    let response = json!({"graphs":[{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567890","id":""}]});
    scenario
        .apply(&catalog, root, lease, &request, response)
        .await?;
    let scenario = scenario.reopen().await?;
    assert_eq!(
        scenario
            .stored("AWS::NeptuneGraph::PrivateGraphEndpoint")
            .await?,
        (BTreeSet::new(), BTreeSet::new())
    );
    let pending = scenario
        .store
        .claim(&scenario.sweep, 20_000, 10_000)
        .await?;
    assert!(
        pending.is_none(),
        "empty request parent scheduled child work"
    );
    Ok(())
}
