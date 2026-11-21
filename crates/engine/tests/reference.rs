use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::reference::{self, REFERENCE_RELATION, REFERENCE_SOURCE, ReferenceConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ACCOUNT: &str = "111111111111";
const REGION: &str = "us-east-1";
const SCAN: &str = "reference-scan";

#[path = "reference/exact_arn.rs"]
mod exact_arn;
#[path = "reference/exact_match_scope.rs"]
mod exact_match_scope;
#[path = "reference/identifier_floor.rs"]
mod identifier_floor;
#[path = "reference/identifier_scope.rs"]
mod identifier_scope;
#[path = "reference/self_reference.rs"]
mod self_reference;

fn resource(id: &str, kind: &str, properties: Value) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: ACCOUNT.to_owned(),
        region: REGION.to_owned(),
        kind: kind.to_owned(),
        name: None,
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 7,
    }
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
                    collector: "reference_fixture".to_owned(),
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

fn property<'a>(edge: &'a inventory_core::Edge, key: &str) -> Option<&'a str> {
    edge.properties.get(key).and_then(Value::as_str)
}

#[tokio::test]
async fn arn_reference_links_s3_bucket_to_kms_key() -> TestResult {
    let key = "arn:aws:kms:us-east-1:111111111111:key/1234abcd";
    let bucket = "arn:aws:s3:::reference-bucket";
    let store = store_with(vec![
        resource(
            bucket,
            "aws_s3_bucket",
            json!({"provider_properties":{"BucketEncryption":{"ServerSideEncryptionConfiguration":[
                {"ServerSideEncryptionByDefault":{"KMSMasterKeyID":key,"SSEAlgorithm":"aws:kms"}}
            ]}}}),
        ),
        resource(
            key,
            "aws_kms_key",
            json!({"resource_identifier":"1234abcd"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, bucket);
    assert_eq!(edge.to_id, key);
    assert_eq!(edge.relation, REFERENCE_RELATION);
    assert_eq!(edge.source, Source::Direct);
    assert_eq!(property(edge, "reference"), Some("arn"));
    assert_eq!(property(edge, "source"), Some(REFERENCE_SOURCE));
    assert_eq!(
        property(edge, "path"),
        Some(
            "/properties/provider_properties/BucketEncryption/ServerSideEncryptionConfiguration/0/ServerSideEncryptionByDefault/KMSMasterKeyID"
        )
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn identifier_reference_links_subnet_to_vpc_arn() -> TestResult {
    let vpc = "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-0123";
    let subnet = "arn:aws:ec2:us-east-1:111111111111:subnet/subnet-0456";
    let store = store_with(vec![
        resource(
            vpc,
            "aws_ec2_vpc",
            json!({"resource_identifier":"vpc-0123","vpc_id":"vpc-0123","cidr_block":"10.0.0.0/16"}),
        ),
        resource(
            subnet,
            "aws_ec2_subnet",
            json!({"resource_identifier":"subnet-0456","vpc_id":"vpc-0123"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, subnet);
    assert_eq!(edge.to_id, vpc);
    assert_eq!(property(edge, "reference"), Some("identifier"));
    assert_eq!(property(edge, "path"), Some("/properties/vpc_id"));
    assert_eq!(property(edge, "source"), Some(REFERENCE_SOURCE));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn unique_primary_identifier_resolves_across_kinds() -> TestResult {
    let target = "arn:aws:service:us-east-1:111111111111:widget/unique-target";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer";
    let store = store_with(vec![
        resource(
            target,
            "aws_service_widget",
            json!({"resource_identifier":"unique-target"}),
        ),
        resource(
            source,
            "aws_service_consumer",
            json!({"linked_widget":"unique-target"}),
        ),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(result.summary.edges_emitted, 1);
    let edge = result.edges.first().ok_or("missing edge")?;
    assert_eq!(edge.from_id, source);
    assert_eq!(edge.to_id, target);
    assert_eq!(property(edge, "reference"), Some("identifier"));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn ambiguous_identifier_emits_nothing_and_is_counted() -> TestResult {
    let first = "arn:aws:service:us-east-1:111111111111:widget/dup-001";
    let second = "arn:aws:service:us-east-1:111111111111:other/dup-001";
    let source = "arn:aws:service:us-east-1:111111111111:widget/consumer-9";
    let duplicate = "dup-ident-001";
    let store = store_with(vec![
        resource(
            first,
            "aws_service_widget",
            json!({"resource_identifier":duplicate}),
        ),
        resource(
            second,
            "aws_service_other",
            json!({"resource_identifier":duplicate}),
        ),
        resource(source, "aws_service_consumer", json!({"linked":duplicate})),
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
async fn replaying_the_same_scan_is_idempotent() -> TestResult {
    let vpc = "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-0123";
    let subnet = "arn:aws:ec2:us-east-1:111111111111:subnet/subnet-0456";
    let store = store_with(vec![
        resource(
            vpc,
            "aws_ec2_vpc",
            json!({"resource_identifier":"vpc-0123"}),
        ),
        resource(subnet, "aws_ec2_subnet", json!({"vpc_id":"vpc-0123"})),
    ])
    .await?;
    let first = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    let second = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(first.edges, second.edges);
    assert_eq!(first.summary, second.summary);
    assert_eq!(first.summary.edges_emitted, 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn deep_payload_is_truncated_without_inventing_an_edge() -> TestResult {
    let key = "arn:aws:kms:us-east-1:111111111111:key/deep";
    let bucket = "arn:aws:s3:::deep-bucket";
    let mut nested = json!(key);
    for _ in 0..12 {
        nested = json!({"next": nested});
    }
    let store = store_with(vec![
        resource(bucket, "aws_s3_bucket", json!({"deep": nested})),
        resource(key, "aws_kms_key", json!({"resource_identifier":"deep"})),
    ])
    .await?;
    let result = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert!(result.edges.is_empty());
    assert!(result.summary.truncated >= 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn oversized_and_node_capped_payloads_are_truncated_without_edges() -> TestResult {
    let key = "arn:aws:kms:us-east-1:111111111111:key/bounded";
    let bucket = "arn:aws:s3:::bounded-bucket";
    let padding = "x".repeat(512);
    let oversized = store_with(vec![
        resource(
            bucket,
            "aws_s3_bucket",
            json!({"padding": padding, "key": key}),
        ),
        resource(key, "aws_kms_key", json!({"resource_identifier":"bounded"})),
    ])
    .await?;
    let result = reference::resolve_scan(
        &oversized,
        SCAN,
        &ReferenceConfig {
            max_properties_bytes: 128,
            ..ReferenceConfig::default()
        },
    )
    .await?;
    assert!(result.edges.is_empty());
    assert!(result.summary.truncated >= 1);
    oversized.close().await;

    let capped = store_with(vec![
        resource(bucket, "aws_s3_bucket", json!({"nest":{"inner":key}})),
        resource(key, "aws_kms_key", json!({"resource_identifier":"bounded"})),
    ])
    .await?;
    let result = reference::resolve_scan(
        &capped,
        SCAN,
        &ReferenceConfig {
            max_nodes_per_resource: 2,
            ..ReferenceConfig::default()
        },
    )
    .await?;
    assert!(result.edges.is_empty());
    assert!(result.summary.truncated >= 1);
    capped.close().await;
    Ok(())
}
