//! Behavior: a resource naming itself through its exact ARN or its primary
//! identifier yields no self-edge and no ambiguity or unresolved accounting.
use super::{SCAN, TestResult, resource, store_with};
use inventory_engine::graph::reference::{self, ReferenceConfig};
use serde_json::json;

#[tokio::test]
async fn exact_arn_self_reference_emits_no_edge_and_no_counters() -> TestResult {
    let id = "arn:aws:service:us-east-1:111111111111:widget/self-arn-reference";
    let store = store_with(vec![resource(
        id,
        "aws_service_widget",
        json!({"self": id}),
    )])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.edges_emitted, 0);
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn identifier_self_reference_emits_no_edge_and_no_counters() -> TestResult {
    let id = "arn:aws:service:us-east-1:111111111111:widget/self-identifier";
    let store = store_with(vec![resource(
        id,
        "aws_service_widget",
        json!({"resource_identifier": "self-ident-value", "linked": "self-ident-value"}),
    )])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.edges_emitted, 0);
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}
