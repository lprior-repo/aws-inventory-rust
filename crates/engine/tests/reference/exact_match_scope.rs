use super::{SCAN, TestResult, resource, store_with};
use inventory_engine::graph::reference::{self, ReferenceConfig};
use serde_json::json;

const FOREIGN_ACCOUNT: &str = "222222222222";

#[tokio::test]
async fn same_identifier_in_multiple_kinds_is_ambiguous() -> TestResult {
    let widget = "arn:aws:service:us-east-1:111111111111:widget/shared-id-01";
    let other = "arn:aws:service:us-east-1:111111111111:other/shared-id-01";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer";
    let store = store_with(vec![
        resource(
            widget,
            "aws_service_widget",
            json!({"resource_identifier": "shared-id-01"}),
        ),
        resource(
            other,
            "aws_service_other",
            json!({"resource_identifier": "shared-id-01"}),
        ),
        resource(
            source,
            "aws_service_consumer",
            json!({"linked": "shared-id-01"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.ambiguous, 1);
    assert_eq!(result.summary.edges_emitted, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn exact_arn_never_ambiguous_across_kinds() -> TestResult {
    let target = "arn:aws:service:us-east-1:111111111111:widget/exact-match-01";
    let decoy = "arn:aws:service:us-east-1:111111111111:other/decoy-01";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer";
    let store = store_with(vec![
        resource(target, "aws_service_widget", json!({})),
        resource(
            decoy,
            "aws_service_other",
            json!({"resource_identifier": "exact-match-01"}),
        ),
        resource(source, "aws_service_consumer", json!({"linked": target})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(result.summary.ambiguous, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn same_identifier_across_accounts_never_resolves() -> TestResult {
    let foreign = "arn:aws:service:us-east-1:222222222222:widget/shared-acct-01";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer";
    let mut target = resource(
        foreign,
        "aws_service_widget",
        json!({"resource_identifier": "shared-acct-01"}),
    );
    target.account_id = FOREIGN_ACCOUNT.to_owned();
    let store = store_with(vec![
        target,
        resource(
            source,
            "aws_service_consumer",
            json!({"linked": "shared-acct-01"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn same_identifier_across_regions_never_resolves() -> TestResult {
    let foreign = "arn:aws:service:eu-west-1:111111111111:widget/shared-reg-01";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer";
    let mut target = resource(
        foreign,
        "aws_service_widget",
        json!({"resource_identifier": "shared-reg-01"}),
    );
    target.region = "eu-west-1".to_owned();
    let store = store_with(vec![
        target,
        resource(
            source,
            "aws_service_consumer",
            json!({"linked": "shared-reg-01"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    store.close().await;
    Ok(())
}
