use anyhow::{Context, Result};
use futures_util::{TryStreamExt, stream};
use inventory_core::{
    CollectionEvent, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

const DEMO_TIME: i64 = 1_794_484_800_000;
const DEMO_SCAN: &str = "synthetic-demo-001";
const MAX_EVENTS: u64 = 1_000_000;
const MAX_EVENT_BYTES: u64 = 16_777_216;
type EventCursor = (BufReader<File>, u64);
type EventStep = Option<(CollectionEvent, EventCursor)>;

pub fn now_ms() -> Result<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

fn resource(kind: &str, id: &str, properties: Value) -> Resource {
    Resource {
        id: id.into(),
        account_id: "111122223333".into(),
        region: "us-east-1".into(),
        kind: kind.into(),
        name: Some("synthetic-example".into()),
        properties,
        tags: BTreeMap::from([("Environment".into(), "demo".into())]),
        source: Source::Direct,
        observed_at: DEMO_TIME,
    }
}

fn resources() -> Vec<Resource> {
    vec![
        resource(
            "aws_vpc",
            "arn:aws:ec2:us-east-1:111122223333:vpc/vpc-demo",
            json!({"vpc_id":"vpc-demo","cidr_block":"10.20.0.0/16","state":"available","is_default":false}),
        ),
        resource(
            "aws_vpc_subnet",
            "arn:aws:ec2:us-east-1:111122223333:subnet/subnet-demo",
            json!({"subnet_id":"subnet-demo","vpc_id":"vpc-demo","cidr_block":"10.20.1.0/24"}),
        ),
        resource(
            "aws_ec2_instance",
            "arn:aws:ec2:us-east-1:111122223333:instance/i-demo",
            json!({"instance_id":"i-demo","vpc_id":"vpc-demo","subnet_id":"subnet-demo","instance_type":"m7i.large","state":"running","private_ip_address":"10.20.1.10"}),
        ),
        resource(
            "aws_dynamodb_table",
            "arn:aws:dynamodb:us-east-1:111122223333:table/demo",
            json!({"table_name":"demo","table_status":"ACTIVE","billing_mode":"PAY_PER_REQUEST"}),
        ),
        resource(
            "aws_lambda_function",
            "arn:aws:lambda:us-east-1:111122223333:function:demo",
            json!({"function_name":"demo","runtime":"provided.al2023","memory_size":256,"timeout":30,"vpc_id":"vpc-demo"}),
        ),
        resource(
            "aws_ecs_cluster",
            "arn:aws:ecs:us-east-1:111122223333:cluster/demo",
            json!({"cluster_arn":"arn:aws:ecs:us-east-1:111122223333:cluster/demo","cluster_name":"demo","status":"ACTIVE"}),
        ),
        resource(
            "aws_eks_cluster",
            "arn:aws:eks:us-east-1:111122223333:cluster/demo",
            json!({"name":"demo","status":"ACTIVE","version":"synthetic","vpc_id":"vpc-demo"}),
        ),
    ]
}

#[tracing::instrument(skip_all)]
pub async fn create(path: PathBuf) -> Result<Value> {
    let store = Store::open(path).await?;
    let result = populate_demo(&store).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
async fn populate_demo(store: &Store) -> Result<Value> {
    store.begin_scan(DEMO_SCAN, DEMO_TIME).await?;
    stream::iter(resources().into_iter().map(Ok::<_, anyhow::Error>))
        .try_for_each(|resource| store_demo_resource(store, resource))
        .await?;
    let summary = store
        .finish_scan(DEMO_SCAN, ScanStatus::Complete, DEMO_TIME)
        .await?;
    Ok(json!({"synthetic":true,"scan_id":DEMO_SCAN,"status":"complete","summary":summary}))
}

#[tracing::instrument(skip_all)]
async fn store_demo_resource(store: &Store, r: Resource) -> Result<()> {
    let vpc = "arn:aws:ec2:us-east-1:111122223333:vpc/vpc-demo";
    let edges = if r.properties.get("vpc_id").is_some() && r.kind != "aws_vpc" {
        vec![Edge {
            from_id: r.id.clone(),
            to_id: vpc.into(),
            relation: "IN_VPC".into(),
            source: Source::Direct,
            observed_at: DEMO_TIME,
            properties: json!({"evidence":"synthetic fixture"}),
        }]
    } else {
        Vec::new()
    };
    store
        .apply_page(
            DEMO_SCAN,
            CollectionPage {
                scope: CollectionScope {
                    account_id: r.account_id.clone(),
                    region: r.region.clone(),
                    collector: r.kind.clone(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources: vec![r],
                edges,
            },
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn ingest(path: PathBuf, input: PathBuf, scan_id: Option<String>) -> Result<Value> {
    let scan = scan_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let store = Store::open(path).await?;
    let result = ingest_open(&store, &scan, &input).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
async fn ingest_open(store: &Store, scan: &str, input: &Path) -> Result<Value> {
    store.begin_scan(scan, now_ms()?).await?;
    let result = read_events(store, scan, input).await;
    finish_ingest(store, scan, result).await
}

#[tracing::instrument(skip_all)]
async fn finish_ingest(store: &Store, scan: &str, result: Result<()>) -> Result<Value> {
    let status = if result.is_ok() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };
    let summary = store.finish_scan(scan, status, now_ms()?).await?;
    result?;
    Ok(json!({"scan_id":scan,"status":store.scan_status(scan).await?.as_str(),"summary":summary}))
}

#[tracing::instrument(skip_all)]
async fn read_events(store: &Store, scan: &str, input: &Path) -> Result<()> {
    let reader = BufReader::new(File::open(input).await?);
    stream::try_unfold((reader, 0_u64), next_event)
        .try_for_each(|event| persist_event(store, scan, event))
        .await
}

#[tracing::instrument(skip_all)]
async fn next_event((mut reader, seen): EventCursor) -> Result<EventStep> {
    let mut line = Vec::new();
    let count = (&mut reader)
        .take(MAX_EVENT_BYTES)
        .read_until(b'\n', &mut line)
        .await?;
    if count == 0 {
        return Ok(None);
    }
    anyhow::ensure!(
        seen < MAX_EVENTS,
        "event ceiling reached; import is incomplete"
    );
    anyhow::ensure!(
        u64::try_from(count)? < MAX_EVENT_BYTES,
        "event exceeds input byte limit"
    );
    let event =
        serde_json::from_slice::<CollectionEvent>(&line).context("invalid inventory event")?;
    Ok(Some((event, (reader, seen.saturating_add(1)))))
}

#[tracing::instrument(skip_all)]
pub async fn persist_event(store: &Store, scan: &str, event: CollectionEvent) -> Result<()> {
    match event {
        CollectionEvent::Page(page) => store.apply_page(scan, page).await?,
        CollectionEvent::Failure(failure) => store.record_failure(scan, &failure).await?,
    }
    Ok(())
}
