//! Public projection contracts: validate complete envelopes before connecting,
//! bound large exports, and never acknowledge a server-rejected batch.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::graph::{self, GraphBatch, GraphError};
use inventory_engine::store::Store;
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SCAN: &str = "projection-contract";
const QUERY_LIMIT: usize = 2_097_152;

fn resource_id(index: usize) -> String {
    format!("arn:aws:ec2:us-east-1:123456789012:instance/i-contract-{index}")
}

fn page(count: usize, metadata: &str) -> CollectionPage {
    CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "projection-fixture".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 1,
        next_token: None,
        resources: (0..count)
            .map(|index| Resource {
                id: resource_id(index),
                account_id: "123456789012".into(),
                region: "us-east-1".into(),
                kind: "aws_ec2_instance".into(),
                name: Some(format!("fixture-{index}")),
                properties: json!({"metadata": metadata}),
                tags: BTreeMap::new(),
                source: Source::Direct,
                observed_at: 1,
            })
            .collect(),
        edges: vec![Edge {
            from_id: resource_id(0),
            to_id: "arn:aws:ec2:us-east-1:123456789012:vpc/vpc-uncollected".into(),
            relation: "IN_VPC".into(),
            properties: json!({}),
            source: Source::Direct,
            observed_at: 1,
        }],
    }
}

async fn frozen_store(
    path: &Path,
    count: usize,
    metadata: &str,
) -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::open(path).await?;
    store.begin_scan(SCAN, 1).await?;
    store.apply_page(SCAN, page(count, metadata)).await?;
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    Ok(store)
}

async fn read_batches(path: &Path) -> Result<Vec<GraphBatch>, Box<dyn std::error::Error>> {
    tokio::fs::read_to_string(path)
        .await?
        .lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

async fn write_batches(path: &Path, batches: &[GraphBatch]) -> TestResult {
    let mut text = batches
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    text.push('\n');
    tokio::fs::write(path, text).await?;
    Ok(())
}

#[tokio::test]
async fn graph_names_are_validated_against_an_existing_frozen_scan() -> TestResult {
    let folder = tempfile::tempdir()?;
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 1, "small").await?;
    let invalid = [
        String::new(),
        "é".into(),
        "graph;delete".into(),
        "graph name".into(),
        "path/name".into(),
        "contains\0nul".into(),
        "a".repeat(129),
        "b".repeat(130),
    ];
    for (index, name) in invalid.iter().enumerate() {
        let output = folder.path().join(format!("invalid-{index}.jsonl"));
        let result = graph::export_snapshot(&store, SCAN, name, &output).await;
        assert!(matches!(result, Err(GraphError::Invalid(_))));
        assert!(!output.exists());
        assert!(
            !folder
                .path()
                .join(format!("invalid-{index}.jsonl.partial"))
                .exists()
        );
    }
    for (index, name) in ["A".to_owned(), "_".into(), "-".into(), "a".repeat(128)]
        .iter()
        .enumerate()
    {
        let output = folder.path().join(format!("valid-{index}.jsonl"));
        let summary = graph::export_snapshot(&store, SCAN, name, &output).await?;
        assert_eq!(summary.graph, *name);
        let batches = read_batches(&output).await?;
        assert!(batches.iter().all(|batch| batch.graph == *name));
        assert_eq!(
            batches.last().map(|batch| batch.stage.as_str()),
            Some("end")
        );
    }
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn metadata_heavy_resources_split_by_bytes_without_losing_rows() -> TestResult {
    let folder = tempfile::tempdir()?;
    // Twelve ordinary 256 KiB property values exceed one graph batch while
    // remaining well below the per-resource and per-page inventory limits.
    let metadata = "x".repeat(262_144);
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 12, &metadata).await?;
    let output = folder.path().join("large.jsonl");
    let summary = graph::export_snapshot(&store, SCAN, "large_metadata", &output).await?;
    let batches = read_batches(&output).await?;
    let resources = batches
        .iter()
        .filter(|batch| batch.stage == "resources")
        .collect::<Vec<_>>();
    assert!(resources.len() >= 2);
    assert_eq!(resources.iter().map(|batch| batch.rows).sum::<u64>(), 12);
    assert!(
        batches
            .iter()
            .all(|batch| batch.rows <= 256 && batch.query.len() <= QUERY_LIMIT)
    );
    for index in 0..12 {
        let field = format!("id:\"{}\"", resource_id(index));
        assert_eq!(
            resources
                .iter()
                .filter(|batch| batch.query.contains(&field))
                .count(),
            1
        );
    }
    assert_eq!(summary.resources, 12);
    assert_eq!(summary.edges, 1);
    assert_eq!(u64::try_from(batches.len())?, summary.batches);
    store.close().await;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Corruption {
    MixedGraph,
    MixedScan,
    SequenceGap,
    Version,
    QueryBytes,
    Rows,
    FirstStage,
    SecondStage,
    RepeatedIndex,
    BackwardStage,
    IndexRows,
    BeginRows,
    EndRows,
    TrailingBatch,
    MissingEnd,
    UnknownStage,
}

fn stage_mut<'a>(
    batches: &'a mut [GraphBatch],
    stage: &str,
) -> Result<&'a mut GraphBatch, Box<dyn std::error::Error>> {
    batches
        .iter_mut()
        .find(|batch| batch.stage == stage)
        .ok_or_else(|| format!("fixture lacks {stage} stage").into())
}

fn corrupt(batches: &mut Vec<GraphBatch>, change: Corruption) -> TestResult {
    match change {
        Corruption::MixedGraph => stage_mut(batches, "resources")?.graph = "other_graph".into(),
        Corruption::MixedScan => stage_mut(batches, "resources")?.scan_id = "other_scan".into(),
        Corruption::SequenceGap => {
            let batch = stage_mut(batches, "resources")?;
            batch.sequence = batch.sequence.saturating_add(2);
        }
        Corruption::Version => stage_mut(batches, "resources")?.version = 2,
        Corruption::QueryBytes => {
            stage_mut(batches, "resources")?.query = "x".repeat(QUERY_LIMIT.saturating_add(1));
        }
        Corruption::Rows => stage_mut(batches, "resources")?.rows = 257,
        Corruption::FirstStage => stage_mut(batches, "index")?.stage = "begin".into(),
        Corruption::SecondStage => stage_mut(batches, "begin")?.stage = "resources".into(),
        Corruption::RepeatedIndex => stage_mut(batches, "resources")?.stage = "index".into(),
        Corruption::BackwardStage => {
            let resource = batches
                .iter()
                .position(|batch| batch.stage == "resources")
                .ok_or("fixture lacks resources")?;
            let edge = batches
                .iter()
                .position(|batch| batch.stage == "edges")
                .ok_or("fixture lacks edges")?;
            batches.get_mut(resource).ok_or("resource index")?.stage = "edges".into();
            batches.get_mut(edge).ok_or("edge index")?.stage = "resources".into();
        }
        Corruption::IndexRows => stage_mut(batches, "index")?.rows = 1,
        Corruption::BeginRows => stage_mut(batches, "begin")?.rows = 1,
        Corruption::EndRows => stage_mut(batches, "end")?.rows = 1,
        Corruption::TrailingBatch => {
            let mut trailing = batches.last().ok_or("fixture is empty")?.clone();
            trailing.sequence = trailing.sequence.saturating_add(1);
            batches.push(trailing);
        }
        Corruption::MissingEnd => {
            batches.pop().ok_or("fixture is empty")?;
        }
        Corruption::UnknownStage => stage_mut(batches, "resources")?.stage = "unknown".into(),
    }
    Ok(())
}

#[tokio::test]
async fn malformed_envelopes_fail_before_connecting_or_creating_acknowledgements() -> TestResult {
    let folder = tempfile::tempdir()?;
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 256, "small").await?;
    let original = folder.path().join("original.jsonl");
    graph::export_snapshot(&store, SCAN, "envelopes", &original).await?;
    let batches = read_batches(&original).await?;
    assert!(
        batches
            .iter()
            .any(|batch| batch.stage == "resources" && batch.rows == 256)
    );
    let valid_ack = folder.path().join("valid-ack.sqlite");
    let valid = graph::push_file(original, valid_ack.clone(), "not-a-redis-url").await;
    assert!(matches!(valid, Err(GraphError::Redis(_))));
    assert!(!valid_ack.exists());

    // Whitespace preserves the generated Cypher while reaching the advertised
    // 2 MiB query limit exactly. Validation must accept this inclusive boundary.
    let mut padded = batches.clone();
    let query = &mut stage_mut(&mut padded, "resources")?.query;
    query.push_str(&" ".repeat(QUERY_LIMIT.saturating_sub(query.len())));
    let padded_path = folder.path().join("padded.jsonl");
    write_batches(&padded_path, &padded).await?;
    let padded_ack = folder.path().join("padded-ack.sqlite");
    let valid = graph::push_file(padded_path, padded_ack.clone(), "not-a-redis-url").await;
    assert!(matches!(valid, Err(GraphError::Redis(_))));
    assert!(!padded_ack.exists());

    for (index, change) in [
        Corruption::MixedGraph,
        Corruption::MixedScan,
        Corruption::SequenceGap,
        Corruption::Version,
        Corruption::QueryBytes,
        Corruption::Rows,
        Corruption::FirstStage,
        Corruption::SecondStage,
        Corruption::RepeatedIndex,
        Corruption::BackwardStage,
        Corruption::IndexRows,
        Corruption::BeginRows,
        Corruption::EndRows,
        Corruption::TrailingBatch,
        Corruption::MissingEnd,
        Corruption::UnknownStage,
    ]
    .into_iter()
    .enumerate()
    {
        let mut damaged = batches.clone();
        corrupt(&mut damaged, change)?;
        let input = folder.path().join(format!("damaged-{index}.jsonl"));
        let ack = folder.path().join(format!("ack-{index}.sqlite"));
        write_batches(&input, &damaged).await?;
        let outcome = graph::push_file(input, ack.clone(), "not-a-redis-url").await;
        assert!(
            matches!(&outcome, Err(GraphError::Invalid(_))),
            "{change:?}: {outcome:?}"
        );
        assert!(
            !ack.exists(),
            "{change:?} created an acknowledgement database"
        );
    }
    store.close().await;
    Ok(())
}

#[test]
fn public_projection_schema_preserves_its_version_and_bounds() {
    assert_eq!(
        graph::graph_batch_schema_version(),
        json!({
            "version": 1,
            "max_rows": 256,
            "max_query_bytes": 2_097_152,
        })
    );
}

async fn acknowledged_sequences(path: &Path) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(path).read_only(true))
        .await?;
    let rows = sqlx::query_scalar("SELECT sequence FROM projection_ack ORDER BY sequence")
        .fetch_all(&pool)
        .await;
    pool.close().await;
    Ok(rows?)
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

async fn rejection_assertions(
    connection: &mut redis::aio::MultiplexedConnection,
    graph_name: &str,
    input: &Path,
    checkpoint: &Path,
    url: &str,
) -> TestResult {
    for _ in 0..2 {
        let result = graph::push_file(input.into(), checkpoint.into(), url).await;
        if !matches!(&result, Err(GraphError::Redis(_))) {
            return Err(format!("server-rejected batch did not fail delivery: {result:?}").into());
        }
        if acknowledged_sequences(checkpoint).await? != vec![0, 1] {
            return Err(
                "a rejected batch was acknowledged or prior acknowledgements were lost".into(),
            );
        }
        let marker: redis::Value = redis::cmd("GRAPH.QUERY")
            .arg(graph_name)
            .arg("MATCH (s:InventoryScan) RETURN s.state")
            .query_async(connection)
            .await?;
        if first_cell(&marker)? != &redis::Value::BulkString(b"loading".to_vec()) {
            return Err("a rejected plan was marked complete".into());
        }
        let count: redis::Value = redis::cmd("GRAPH.QUERY")
            .arg(graph_name)
            .arg("MATCH (r:Resource) RETURN count(r)")
            .query_async(connection)
            .await?;
        if first_cell(&count)? != &redis::Value::Int(0) {
            return Err("resource rows appeared after their batch was rejected".into());
        }
    }
    Ok(())
}

async fn server_rejection_case() -> TestResult {
    let url = std::env::var("FALKORDB_TEST_URL")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let name = format!("inventory_rejection_{}_{nonce}", std::process::id());
    let folder = tempfile::tempdir()?;
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 1, "small").await?;
    let input = folder.path().join("rejected.jsonl");
    graph::export_snapshot(&store, SCAN, &name, &input).await?;
    let mut batches = read_batches(&input).await?;
    stage_mut(&mut batches, "resources")?.query = "RETURN definitely_undefined_variable".into();
    write_batches(&input, &batches).await?;
    let mut connection = redis::Client::open(url.as_str())?
        .get_multiplexed_async_connection()
        .await?;
    let result = rejection_assertions(
        &mut connection,
        &name,
        &input,
        &folder.path().join("ack.sqlite"),
        &url,
    )
    .await;
    let cleanup: Result<redis::Value, redis::RedisError> = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async(&mut connection)
        .await;
    store.close().await;
    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Ok(()), Ok(_)) => Ok(()),
    }
}

#[tokio::test]
#[ignore = "requires explicit FALKORDB_TEST_URL; creates and removes an isolated test graph"]
async fn live_falkordb_rejected_batch_is_never_acknowledged_or_finalized() -> TestResult {
    tokio::time::timeout(Duration::from_secs(10), server_rejection_case())
        .await
        .map_err(|_| "engine graph integration test exceeded its 10-second deadline")?
}
