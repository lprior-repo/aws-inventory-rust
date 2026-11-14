use super::{ACCOUNT, SCAN, TestResult, property, resource, store_with};
use inventory_engine::graph::reference::{self, ReferenceConfig};
use serde_json::json;

const FLOOR: usize = 8;

#[tokio::test]
async fn short_identifiers_never_emit_edges_or_counters() -> TestResult {
    let three = "abc";
    let seven = "enumval";
    assert!(three.len() < FLOOR && seven.len() < FLOOR);
    let store = store_with(vec![
        resource(
            "arn:aws:service:us-east-1:111111111111:widget/widget-owner-three",
            "aws_service_widget",
            json!({"resource_identifier":three}),
        ),
        resource(
            "arn:aws:service:us-east-1:111111111111:widget/widget-owner-seven",
            "aws_service_widget",
            json!({"resource_identifier":seven}),
        ),
        resource(
            "arn:aws:service:us-east-1:111111111111:widget/consumer-short",
            "aws_service_consumer",
            json!({"first":three,"second":seven}),
        ),
    ])
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
async fn identifier_at_the_floor_emits_the_weakest_accepted_edge() -> TestResult {
    let identifier = "widget01";
    assert_eq!(identifier.len(), FLOOR);
    let target = "arn:aws:service:us-east-1:111111111111:widget/owner-floor";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer-floor";
    let store = store_with(vec![
        resource(
            target,
            "aws_service_widget",
            json!({"resource_identifier":identifier}),
        ),
        resource(source, "aws_service_consumer", json!({"linked":identifier})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 0);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(property(edge, "reference"), Some("identifier"));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn arn_component_floor_is_unchanged_by_the_identifier_floor() -> TestResult {
    let target = "arn:aws:sns:us-east-1:111111111111:abc";
    let source = "arn:aws:sqs:us-east-1:111111111111:consumer-component";
    let reference_arn = "arn:aws:sqs:us-east-1:111111111111:abc";
    assert_eq!(reference_arn.rsplit(':').next(), Some("abc"));
    let store = store_with(vec![
        resource(target, "aws_sns_topic", json!({})),
        resource(source, "aws_sqs_queue", json!({"linked_arn":reference_arn})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(property(edge, "reference"), Some("identifier"));
    assert_eq!(result.summary.ambiguous, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn floor_matches_across_another_account_never_emit() -> TestResult {
    let mut foreign = resource(
        "arn:aws:service:us-east-1:222222222222:widget/owner-floor",
        "aws_service_widget",
        json!({"resource_identifier":"widget01"}),
    );
    foreign.account_id = "222222222222".to_owned();
    let source = format!("arn:aws:service:us-east-1:{ACCOUNT}:widget/consumer-floor");
    let store = store_with(vec![
        foreign,
        resource(
            &source,
            "aws_service_consumer",
            json!({"linked":"widget01"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.ambiguous, 0);
    assert_eq!(result.summary.references_unresolved, 1);
    store.close().await;
    Ok(())
}
