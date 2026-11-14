use super::{SCAN, TestResult, property, resource, store_with};
use inventory_engine::graph::reference::{self, ReferenceConfig};
use serde_json::json;

#[tokio::test]
async fn exact_arn_resolves_when_the_trailing_component_is_too_short() -> TestResult {
    let target = "arn:aws:service:us-east-1:111111111111:widget/xy";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer-exact";
    let store = store_with(vec![
        resource(target, "aws_service_widget", json!({})),
        resource(source, "aws_service_consumer", json!({"linked": target})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(property(edge, "reference"), Some("arn"));
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn exact_arn_resolves_despite_an_ambiguous_trailing_component() -> TestResult {
    let target = "arn:aws:service:us-east-1:111111111111:widget/exact-arn-target";
    let decoy = "arn:aws:service:us-east-1:111111111111:other/decoy-target";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer-exact";
    let store = store_with(vec![
        resource(target, "aws_service_widget", json!({})),
        resource(
            decoy,
            "aws_service_other",
            json!({"resource_identifier": "exact-arn-target"}),
        ),
        resource(source, "aws_service_consumer", json!({"linked": target})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(property(edge, "reference"), Some("arn"));
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}
