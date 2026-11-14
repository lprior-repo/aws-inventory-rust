use super::{Direction, TraversalOptions, encoded_len, finalized, literal, query_rows};
use anyhow::{Context, Result};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::{ScanStatus, Source, validate_identifier};
use inventory_engine::store::Store;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};

const GRAPH_DETAIL_BYTES: usize = 14_680_064;
type EdgeKey = (String, String, String);

#[tracing::instrument(skip_all)]
pub(super) async fn report(
    store: &Store,
    scan: &str,
    start: &str,
    target: Option<&str>,
    options: TraversalOptions,
) -> Result<Value> {
    super::validate_traversal(scan, start, options)?;
    if let Some(target) = target {
        validate_identifier(target, "path target")?;
    }
    let status = finalized(store, scan).await?;
    let seed = seed(store, scan, start).await?;
    let walk = walk(store, scan, Walk::new(seed, options, target)?).await?;
    let path = target.map(|target| walk.path(start, target)).transpose()?;
    let partial = status != ScanStatus::Complete || walk.truncation.any();
    Ok(json!({
        "status":if partial {"partial"} else {"complete"},"scan_id":scan,
        "scan_status":status.as_str(),"start_id":start,"target_id":target,"limits":options,
        "truncated":walk.truncation.any(),"truncation":walk.truncation,
        "expanded_nodes":walk.expanded_nodes,"depth_boundary_nodes":walk.depth_boundary_nodes,
        "inspected_edges":walk.inspected_edges,"inspected_edge_limit":walk.inspection_limit,
        "detail_byte_limit":GRAPH_DETAIL_BYTES,
        "nodes":walk.nodes.values().collect::<Vec<_>>(),
        "relationships":walk.edges.values().collect::<Vec<_>>(),"path":path,
        "semantics":{
            "identity":"exact identity in the selected scan; no cross-scan endpoint substitution",
            "distance":"breadth-first relationship hops in the requested direction",
            "incoming":"reverse references are potential dependents; they are not proof of operational blast radius",
            "edges":"relationships encountered while expanding nodes below max_depth; boundary-only edges are not enumerated",
            "unresolved":"referenced endpoint was not observed in this scan; it is retained without fabricated metadata",
            "limits":"depth defines the requested radius; node, edge, byte and inspection exhaustion are explicit truncation",
            "path":"one deterministic shortest observed path when found without work truncation; not effective network or IAM reachability",
            "no_path":"a missing path is scoped to this snapshot, selected direction and limits; it does not prove isolation"
        }
    }))
}

#[derive(Deserialize)]
struct SeedRow {
    id: String,
    kind: String,
    name: Option<String>,
}

#[tracing::instrument(skip_all)]
async fn seed(store: &Store, scan: &str, start: &str) -> Result<Node> {
    let sql = format!(
        "SELECT id,kind,name FROM scan_resources WHERE scan_id={} AND id={} LIMIT 1",
        literal(scan),
        literal(start)
    );
    let rows = query_rows(store, &sql, 1).await?;
    if let Some(row) = rows.into_iter().next() {
        let row: SeedRow = serde_json::from_value(row).context("decode start resource")?;
        return Ok(Node {
            id: row.id,
            kind: Some(row.kind),
            name: row.name,
            observed: true,
            distance: 0,
        });
    }
    let sql = format!(
        "SELECT 1 AS present FROM scan_edges WHERE scan_id={} AND (from_id={} OR to_id={}) LIMIT 1",
        literal(scan),
        literal(start),
        literal(start)
    );
    anyhow::ensure!(
        !query_rows(store, &sql, 1).await?.is_empty(),
        "start identity is neither observed nor referenced in this scan"
    );
    Ok(Node {
        id: start.into(),
        kind: None,
        name: None,
        observed: false,
        distance: 0,
    })
}

#[tracing::instrument(skip_all)]
async fn walk(store: &Store, scan: &str, state: Walk) -> Result<Walk> {
    let max_nodes = state.options.max_nodes;
    stream::iter(0..max_nodes)
        .map(Ok::<_, anyhow::Error>)
        .try_fold(state, |state, _| expand(store, scan, state))
        .await
}

#[tracing::instrument(skip_all)]
async fn expand(store: &Store, scan: &str, mut state: Walk) -> Result<Walk> {
    tokio::task::coop::consume_budget().await;
    if state.stopped() {
        return Ok(state);
    }
    let Some(current) = state.queue.pop_front() else {
        return Ok(state);
    };
    let distance = state
        .nodes
        .get(&current)
        .context("queued node missing")?
        .distance;
    if distance >= state.options.max_depth {
        state.depth_boundary_nodes = state.depth_boundary_nodes.saturating_add(1);
        return Ok(state);
    }
    state.expanded_nodes = state.expanded_nodes.saturating_add(1);
    let next_distance = distance.checked_add(1).context("graph depth overflow")?;
    let probe = state
        .options
        .max_edges
        .checked_add(1)
        .context("edge probe overflow")?;
    let sql = adjacency_sql(scan, &current, state.options.direction, probe);
    let rows = query_rows(store, &sql, probe).await?;
    accept_rows(state, &current, next_distance, rows).await
}

fn adjacency_sql(scan: &str, current: &str, direction: Direction, probe: usize) -> String {
    let id = literal(current);
    let selection = match direction {
        Direction::Outgoing => format!("from_id={id}"),
        Direction::Incoming => format!("to_id={id}"),
        Direction::Both => format!("(from_id={id} OR to_id={id})"),
    };
    format!("SELECT from_id,to_id,relation,source,observed_at,from_kind,from_name,to_kind,to_name,
        source_observed,target_observed,json_extract(properties,'$.owner_resolution') AS owner_resolution
        FROM scan_architecture_edges WHERE scan_id={} AND {selection}
        ORDER BY from_id,to_id,relation LIMIT {probe}", literal(scan))
}

#[tracing::instrument(skip_all)]
async fn accept_rows(
    state: Walk,
    current: &str,
    distance: usize,
    rows: Vec<Value>,
) -> Result<Walk> {
    stream::iter(rows)
        .map(Ok::<_, anyhow::Error>)
        .try_fold(state, |mut state, row| async move {
            tokio::task::coop::consume_budget().await;
            if !state.stopped() {
                let edge = serde_json::from_value(row).context("decode adjacency relationship")?;
                state.accept(current, distance, edge)?;
            }
            Ok(state)
        })
        .await
}

mod model;
use model::Node;
use model::Walk;
