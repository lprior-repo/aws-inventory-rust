use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::reference::{self, ReferenceConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::error::Error;

const SCAN: &str = "reference-demo";
const ACCOUNT: &str = "111111111111";
const REGION: &str = "us-east-1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1);
    let scan = std::env::args().nth(2);
    let persist = std::env::args()
        .nth(3)
        .is_some_and(|value| value == "persist");
    let demo = path.is_none();
    let (store, scan_id) = match (path, scan) {
        (Some(path), Some(scan)) => (Store::open(&path).await?, scan),
        (None, None) => {
            let path = std::env::temp_dir().join("inventory-reference-demo.sqlite");
            match tokio::fs::remove_file(&path).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let store = Store::open(&path).await?;
            seed(&store).await?;
            (store, SCAN.to_owned())
        }
        _ => return Err("provide both a database path and a scan id, or neither".into()),
    };
    let first = reference::resolve_scan(&store, &scan_id, &ReferenceConfig::default()).await?;
    if demo || persist {
        reference::persist_scan(&store, &scan_id, &ReferenceConfig::default()).await?;
    }
    let second = reference::resolve_scan(&store, &scan_id, &ReferenceConfig::default()).await?;
    let persisted_references = store
        .edges(&scan_id)
        .await?
        .iter()
        .filter(|edge| edge.relation == reference::REFERENCE_RELATION)
        .count();
    let replay_stable = first == second;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "scan_id": scan_id,
            "replay_stable": replay_stable,
            "edges_emitted": first.summary.edges_emitted,
            "summary": first.summary,
            "persisted_references": persisted_references,
            "edges": first.edges,
        }))?
    );
    if !replay_stable {
        return Err("replay produced a different edge set".into());
    }
    store.close().await;
    Ok(())
}

async fn seed(store: &Store) -> Result<(), Box<dyn Error>> {
    store.begin_scan(SCAN, 1).await?;
    store
        .apply_page(
            SCAN,
            CollectionPage {
                scope: CollectionScope {
                    account_id: ACCOUNT.to_owned(),
                    region: REGION.to_owned(),
                    collector: "reference_demo".to_owned(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources: demo_resources(),
                edges: Vec::new(),
            },
        )
        .await?;
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    Ok(())
}

fn demo_resources() -> Vec<Resource> {
    let key = "arn:aws:kms:us-east-1:111111111111:key/abcdef01";
    let bucket = "arn:aws:s3:::reference-demo-bucket";
    let vpc = "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-0123";
    let subnet = "arn:aws:ec2:us-east-1:111111111111:subnet/subnet-0456";
    vec![
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
            json!({"resource_identifier":"abcdef01"}),
        ),
        resource(
            vpc,
            "aws_ec2_vpc",
            json!({"resource_identifier":"vpc-0123","vpc_id":"vpc-0123"}),
        ),
        resource(
            subnet,
            "aws_ec2_subnet",
            json!({"resource_identifier":"subnet-0456","vpc_id":"vpc-0123"}),
        ),
        resource(
            "arn:aws:service:us-east-1:111111111111:widget/dup-001",
            "aws_service_widget",
            json!({"resource_identifier":"dup-001"}),
        ),
        resource(
            "arn:aws:service:us-east-1:111111111111:other/dup-001",
            "aws_service_other",
            json!({"resource_identifier":"dup-001"}),
        ),
        resource(
            "arn:aws:service:us-east-1:111111111111:widget/consumer",
            "aws_service_consumer",
            json!({
                "linked_dup":"dup-001",
                "missing_key":"arn:aws:kms:us-east-1:111111111111:key/missing",
            }),
        ),
    ]
}

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
        observed_at: 1,
    }
}
