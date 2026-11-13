use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::{self, GraphBatch},
    store::Store,
};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn page(count: usize) -> CollectionPage {
    let resources = (0..count)
        .map(|index| Resource {
            id: format!("arn:aws:ec2:us-east-1:123456789012:instance/i-{index}"),
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            kind: "aws_ec2_instance".into(),
            name: Some("quote\" newline\n slash\\ 😃".into()),
            properties: json!({"value":"x\"}) MATCH (n) DETACH DELETE n //"}),
            tags: BTreeMap::from([("env".into(), "test".into())]),
            source: Source::Direct,
            observed_at: 1,
        })
        .collect();
    CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "ec2".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 1,
        next_token: None,
        resources,
        edges: vec![Edge {
            from_id: "arn:aws:ec2:us-east-1:123456789012:instance/i-0".into(),
            to_id: "arn:aws:ec2:us-east-1:123456789012:vpc/vpc-unknown".into(),
            relation: "IN_VPC".into(),
            source: Source::Direct,
            observed_at: 1,
            properties: json!({}),
        }],
    }
}

#[tokio::test]
async fn export_is_bounded_deterministic_and_preserves_special_characters() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = Store::open(dir.path().join("source.sqlite")).await?;
    store.begin_scan("scan-one", 1).await?;
    store.apply_page("scan-one", page(600)).await?;
    store
        .finish_scan("scan-one", ScanStatus::Complete, 2)
        .await?;
    let first = dir.path().join("first.jsonl");
    let second = dir.path().join("second.jsonl");
    let summary = graph::export_snapshot(&store, "scan-one", "aws_inventory", &first).await?;
    graph::export_snapshot(&store, "scan-one", "aws_inventory", &second).await?;
    let first_bytes = tokio::fs::read(&first).await?;
    assert_eq!(first_bytes, tokio::fs::read(&second).await?);
    let batches = std::str::from_utf8(&first_bytes)?
        .lines()
        .map(serde_json::from_str::<GraphBatch>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(summary.resources, 600);
    assert_eq!(summary.edges, 1);
    assert_eq!(
        batches
            .iter()
            .filter(|batch| batch.stage == "resources")
            .count(),
        3
    );
    assert!(
        batches
            .iter()
            .all(|batch| batch.rows <= 256 && batch.query.len() <= 2_097_152)
    );
    assert!(
        batches
            .iter()
            .any(|batch| batch.query.contains("\\\"") && batch.query.contains("😃"))
    );
    assert!(
        batches
            .iter()
            .any(|batch| batch.stage == "edges" && batch.query.contains("ON CREATE SET"))
    );
    assert!(
        batches
            .last()
            .is_some_and(|batch| batch.stage == "end" && batch.query.contains("\"complete\""))
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn running_scan_is_rejected_and_partial_scan_is_labeled() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = Store::open(dir.path().join("source.sqlite")).await?;
    store.begin_scan("partial", 1).await?;
    let path = dir.path().join("partial.jsonl");
    assert!(
        graph::export_snapshot(&store, "partial", "graph", &path)
            .await
            .is_err()
    );
    assert!(!path.exists());
    store.apply_page("partial", page(1)).await?;
    store.finish_scan("partial", ScanStatus::Partial, 2).await?;
    graph::export_snapshot(&store, "partial", "graph", &path).await?;
    let lines = tokio::fs::read_to_string(path).await?;
    let last = lines.lines().last().ok_or("missing end marker")?;
    let batch: GraphBatch = serde_json::from_str(last)?;
    assert!(batch.query.contains("\"partial\""));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn truncated_plan_fails_validation_before_network_connection() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = Store::open(dir.path().join("source.sqlite")).await?;
    store.begin_scan("scan", 1).await?;
    store.apply_page("scan", page(1)).await?;
    store.finish_scan("scan", ScanStatus::Complete, 2).await?;
    let input = dir.path().join("graph.jsonl");
    graph::export_snapshot(&store, "scan", "graph", &input).await?;
    let complete = tokio::fs::read_to_string(&input).await?;
    let lines: Vec<_> = complete.lines().collect();
    let truncated = lines
        .iter()
        .take(lines.len().saturating_sub(1))
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(&input, truncated).await?;
    let result = graph::push_file(
        input,
        dir.path().join("checkpoint.sqlite"),
        "not-a-redis-url",
    )
    .await;
    assert!(result.is_err_and(|error| error.to_string().contains("end marker")));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn invalid_graph_name_never_creates_an_output_file() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = Store::open(dir.path().join("source.sqlite")).await?;
    let output = dir.path().join("graph.jsonl");
    assert!(
        graph::export_snapshot(&store, "scan", "x;delete", &output)
            .await
            .is_err()
    );
    assert!(!output.exists());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn graph_export_cannot_replace_the_inventory_database() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("source.sqlite");
    let store = Store::open(&path).await?;
    store.begin_scan("scan", 1).await?;
    store.apply_page("scan", page(1)).await?;
    store.finish_scan("scan", ScanStatus::Complete, 2).await?;
    assert!(
        graph::export_snapshot(&store, "scan", "graph", &path)
            .await
            .is_err()
    );
    store.close().await;
    let reopened = Store::open(&path).await?;
    assert_eq!(reopened.summary("scan").await?.resources, 1);
    reopened.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires explicit FALKORDB_TEST_URL; creates and removes an isolated test graph"]
async fn live_falkordb_replay_and_parameter_round_trip() -> TestResult {
    let url = std::env::var("FALKORDB_TEST_URL")?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let name = format!("inventory_test_{}_{nonce}", std::process::id());
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("inventory.sqlite")).await?;
    store.begin_scan("roundtrip", 1).await?;
    store.apply_page("roundtrip", page(3)).await?;
    store
        .finish_scan("roundtrip", ScanStatus::Complete, 2)
        .await?;
    let plan = directory.path().join("plan.jsonl");
    graph::export_snapshot(&store, "roundtrip", &name, &plan).await?;
    let client = redis::Client::open(url.as_str())?;
    let mut connection = client.get_multiplexed_async_connection().await?;
    let result = live_assertions(&mut connection, &name, &plan, directory.path(), &url).await;
    let cleanup: Result<redis::Value, _> = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async(&mut connection)
        .await;
    store.close().await;
    cleanup?;
    result
}

async fn live_assertions(
    connection: &mut redis::aio::MultiplexedConnection,
    name: &str,
    plan: &std::path::Path,
    directory: &std::path::Path,
    url: &str,
) -> TestResult {
    let checkpoint = directory.join("ack.sqlite");
    let first = graph::push_file(plan.into(), checkpoint.clone(), url).await?;
    let resumed = graph::push_file(plan.into(), checkpoint, url).await?;
    let replayed = graph::push_file(plan.into(), directory.join("fresh.sqlite"), url).await?;
    if first != 5 || resumed != 0 || replayed != 5 {
        return Err("unexpected delivery/replay batch counts".into());
    }
    let nodes = live_query(connection, name, "MATCH (r:Resource) RETURN count(r)").await?;
    let edges = live_query(
        connection,
        name,
        "MATCH ()-[e:RELATES_TO]->() RETURN count(e)",
    )
    .await?;
    if first_cell(&nodes)? != &redis::Value::Int(4) || first_cell(&edges)? != &redis::Value::Int(1)
    {
        return Err("graph replay duplicated, lost or injected records".into());
    }
    let names = live_query(
        connection,
        name,
        "MATCH (r:Resource) WHERE r.unresolved=false RETURN r.name LIMIT 1",
    )
    .await?;
    let expected = redis::Value::BulkString("quote\" newline\n slash\\ 😃".as_bytes().to_vec());
    if first_cell(&names)? != &expected {
        return Err("graph string parameter round-trip mismatch".into());
    }
    let marker = live_query(connection, name, "MATCH (s:InventoryScan) RETURN s.state").await?;
    if first_cell(&marker)? != &redis::Value::BulkString(b"complete".to_vec()) {
        return Err("missing complete marker".into());
    }
    Ok(())
}

async fn live_query(
    connection: &mut redis::aio::MultiplexedConnection,
    name: &str,
    query: &str,
) -> Result<redis::Value, redis::RedisError> {
    redis::cmd("GRAPH.QUERY")
        .arg(name)
        .arg(query)
        .query_async(connection)
        .await
}

fn first_cell(value: &redis::Value) -> Result<&redis::Value, Box<dyn std::error::Error>> {
    if let redis::Value::Array(parts) = value
        && let Some(redis::Value::Array(rows)) = parts.get(1)
        && let Some(redis::Value::Array(cells)) = rows.first()
    {
        return cells
            .first()
            .ok_or_else(|| "missing graph result cell".into());
    }
    Err("unexpected graph response structure".into())
}
