use super::*;
#[tokio::test]
async fn given_a_native_graph_with_empty_request_parent_field_when_child_work_is_attempted_then_no_phantom_endpoint_can_persist()
-> TestResult {
    // The parent ARN is valid; its separate queryable id must not become an empty correlated operand.
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
    let failed_operation = if scenario
        .apply(&catalog, root, lease, &request, response)
        .await
        .is_ok()
    {
        let child = plan(&catalog, "neptune-graph", "ListPrivateGraphEndpoints")?;
        let lease = scenario.claim().await?;
        assert_eq!(
            lease.scope().operation().as_str(),
            "ListPrivateGraphEndpoints"
        );
        // Rejecting an empty HTTP label before projection is also fail-closed behavior.
        if let Ok((request, _)) = scenario.request(child, &lease) {
            let response = page(
                json!({"privateGraphEndpoints":[row("vpcId", "vpc-11111111")]}),
                "nextToken",
                false,
            )?;
            let error = scenario
                .apply(&catalog, child, lease, &request, response)
                .await
                .err()
                .ok_or("empty correlated request parent was committed")?;
            assert!(
                error
                    .to_string()
                    .contains("modeled identity text operand is empty"),
                "{error}"
            );
        }
        "ListPrivateGraphEndpoints"
    } else {
        "ListGraphs"
    };
    let scenario = scenario.reopen().await?;
    assert_eq!(
        scenario
            .stored("AWS::NeptuneGraph::PrivateGraphEndpoint")
            .await?,
        (BTreeSet::new(), BTreeSet::new())
    );
    let retry = scenario
        .store
        .claim(&scenario.sweep, 20_000, 10_000)
        .await?
        .ok_or("empty-parent page checkpoint was lost")?;
    assert_eq!(retry.scope().operation().as_str(), failed_operation);
    assert!(retry.token().is_none());
    Ok(())
}
