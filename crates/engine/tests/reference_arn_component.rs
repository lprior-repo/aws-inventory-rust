//! Behavior: an ARN-valued property whose exact ARN is not an observed
//! resource identity resolves through the ARN's final component, scoped to the
//! ARN's own account and region and only when that component is unique.
use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::reference::{self, ReferenceConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SCAN: &str = "arn-component-scan";
const ACCOUNT: &str = "000000000000";
const REGION: &str = "us-east-1";

fn resource(id: &str, region: &str, kind: &str, properties: Value) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: ACCOUNT.to_owned(),
        region: region.to_owned(),
        kind: kind.to_owned(),
        name: None,
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 1,
    }
}

fn topic(region: &str, suffix: &str) -> Resource {
    resource(
        &format!("awscc:aws:{region}:{ACCOUNT}:AWS::SNS::Topic:{suffix}"),
        region,
        "aws_sns_topic",
        json!({"resource_identifier":"inventory-probe-topic-two"}),
    )
}

fn subscription() -> Resource {
    resource(
        &format!("awscc:aws:{REGION}:{ACCOUNT}:AWS::SNS::Subscription:subscription"),
        REGION,
        "aws_sns_subscription",
        json!({"provider_properties":{
            "Protocol":"sqs",
            "TopicArn":format!("arn:aws:sns:{REGION}:{ACCOUNT}:inventory-probe-topic-two"),
        }}),
    )
}

async fn store_with(resources: Vec<Resource>) -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    store
        .apply_page(
            SCAN,
            CollectionPage {
                scope: CollectionScope {
                    account_id: ACCOUNT.to_owned(),
                    region: REGION.to_owned(),
                    collector: "arn_component_fixture".to_owned(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources,
                edges: Vec::new(),
            },
        )
        .await?;
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    Ok(store)
}

#[tokio::test]
async fn arn_component_resolves_topic_identifier_and_ignores_other_regions() -> TestResult {
    let primary = topic(REGION, "topic-two");
    let subscription = subscription();
    let store = store_with(vec![
        subscription.clone(),
        primary.clone(),
        topic("eu-west-1", "topic-two-decoy"),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, subscription.id);
    assert_eq!(edge.to_id, primary.id);
    assert_eq!(edge.relation, reference::REFERENCE_RELATION);
    assert_eq!(
        edge.properties.get("reference").and_then(Value::as_str),
        Some("identifier")
    );
    assert_eq!(
        edge.properties.get("path").and_then(Value::as_str),
        Some("/properties/provider_properties/TopicArn")
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn arn_component_never_matches_a_resource_in_another_region() -> TestResult {
    let store = store_with(vec![subscription(), topic("eu-west-1", "topic-two-decoy")]).await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    assert_eq!(result.summary.ambiguous, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn arn_component_with_two_targets_is_ambiguous() -> TestResult {
    let store = store_with(vec![
        subscription(),
        topic(REGION, "topic-two-first"),
        topic(REGION, "topic-two-second"),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.ambiguous, 1);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn arn_component_resolution_is_idempotent() -> TestResult {
    let store = store_with(vec![subscription(), topic(REGION, "topic-two")]).await?;
    let first = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    let second = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(first, second);
    assert_eq!(first.summary.edges_emitted, 1);
    store.close().await;
    Ok(())
}

fn workspace_arn(region: &str) -> String {
    format!("arn:aws:aps:{region}:{ACCOUNT}:workspace/ws-12345678")
}

fn workspace(region: &str, suffix: &str) -> Resource {
    resource(
        &format!("awscc:aws:{region}:{ACCOUNT}:AWS::APS::Workspace:{suffix}"),
        region,
        "aws_aps_workspace",
        json!({"resource_identifier":workspace_arn(region)}),
    )
}

fn scraper(workspace_arn: &str) -> Resource {
    resource(
        &format!("awscc:aws:{REGION}:{ACCOUNT}:AWS::APS::Scraper:scraper"),
        REGION,
        "aws_aps_scraper",
        json!({"provider_properties":{"ScraperConfiguration":{"WorkspaceArn":workspace_arn}}}),
    )
}

#[tokio::test]
async fn arn_shaped_primary_identifier_of_synthesized_resource_resolves() -> TestResult {
    let workspace = workspace(REGION, "workspace");
    let scraper = scraper(&workspace_arn(REGION));
    let store = store_with(vec![scraper.clone(), workspace.clone()]).await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, scraper.id);
    assert_eq!(edge.to_id, workspace.id);
    assert_eq!(
        edge.properties.get("reference").and_then(Value::as_str),
        Some("identifier")
    );
    assert_eq!(
        edge.properties.get("path").and_then(Value::as_str),
        Some("/properties/provider_properties/ScraperConfiguration/WorkspaceArn")
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn arn_shaped_primary_identifier_keeps_region_scope_and_ambiguity() -> TestResult {
    // The ARN names us-east-1, but the only owner was observed in eu-west-1.
    let mut mismatched = workspace(REGION, "workspace");
    mismatched.region = "eu-west-1".to_owned();
    let store = store_with(vec![scraper(&workspace_arn(REGION)), mismatched]).await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.references_unresolved, 1);
    store.close().await;

    // Two in-region owners of the same ARN identifier are ambiguous.
    let store = store_with(vec![
        scraper(&workspace_arn(REGION)),
        workspace(REGION, "workspace-first"),
        workspace(REGION, "workspace-second"),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert_eq!(result.summary.ambiguous, 1);
    assert_eq!(result.summary.references_unresolved, 0);
    store.close().await;
    Ok(())
}
