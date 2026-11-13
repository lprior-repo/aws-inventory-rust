//! Deterministic breadth-first traversal with bounded database adjacency reads.
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Node {
    id: String,
    kind: Option<String>,
    name: Option<String>,
    observed: bool,
    distance: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Relationship {
    from_id: String,
    to_id: String,
    relation: String,
    source: Source,
    observed_at: i64,
    from_kind: Option<String>,
    from_name: Option<String>,
    to_kind: Option<String>,
    to_name: Option<String>,
    source_observed: u8,
    target_observed: u8,
    owner_resolution: Option<String>,
}

impl Relationship {
    fn validate(&self) -> Result<()> {
        validate_identifier(&self.from_id, "edge source")?;
        validate_identifier(&self.to_id, "edge target")?;
        validate_identifier(&self.relation, "edge relation")?;
        anyhow::ensure!(
            self.source_observed <= 1 && self.target_observed <= 1,
            "invalid observed-endpoint flag"
        );
        anyhow::ensure!(self.observed_at >= 0, "negative edge observation time");
        Ok(())
    }

    fn key(&self) -> EdgeKey {
        (
            self.from_id.clone(),
            self.to_id.clone(),
            self.relation.clone(),
        )
    }

    fn neighbor<'a>(&'a self, current: &str, direction: Direction) -> Result<&'a str> {
        match direction {
            Direction::Outgoing if self.from_id == current => Ok(&self.to_id),
            Direction::Incoming if self.to_id == current => Ok(&self.from_id),
            Direction::Both if self.from_id == current => Ok(&self.to_id),
            Direction::Both if self.to_id == current => Ok(&self.from_id),
            _ => anyhow::bail!("adjacency row does not match traversal direction"),
        }
    }

    fn endpoint(&self, id: &str, distance: usize) -> Result<Node> {
        if self.from_id == id {
            return Ok(Node {
                id: id.into(),
                kind: self.from_kind.clone(),
                name: self.from_name.clone(),
                observed: self.source_observed == 1,
                distance,
            });
        }
        anyhow::ensure!(self.to_id == id, "endpoint does not belong to edge");
        Ok(Node {
            id: id.into(),
            kind: self.to_kind.clone(),
            name: self.to_name.clone(),
            observed: self.target_observed == 1,
            distance,
        })
    }
}

#[derive(Default, Serialize)]
struct Truncation {
    node_limit: bool,
    edge_limit: bool,
    detail_byte_limit: bool,
    inspected_edge_limit: bool,
}

impl Truncation {
    fn any(&self) -> bool {
        self.node_limit || self.edge_limit || self.detail_byte_limit || self.inspected_edge_limit
    }
}

struct Predecessor {
    from: String,
    edge: EdgeKey,
}

struct Walk {
    nodes: BTreeMap<String, Node>,
    edges: BTreeMap<EdgeKey, Relationship>,
    parents: BTreeMap<String, Predecessor>,
    queue: VecDeque<String>,
    options: TraversalOptions,
    target: Option<String>,
    found: bool,
    expanded_nodes: usize,
    depth_boundary_nodes: usize,
    inspected_edges: usize,
    inspection_limit: usize,
    detail_bytes: usize,
    truncation: Truncation,
}

impl Walk {
    fn new(seed: Node, options: TraversalOptions, target: Option<&str>) -> Result<Self> {
        options.validate()?;
        let mut queue = VecDeque::new();
        queue
            .try_reserve(options.max_nodes)
            .context("reserve traversal queue")?;
        queue.push_back(seed.id.clone());
        let found = target.is_some_and(|target| target == seed.id);
        let detail_bytes = encoded_len(&seed)?;
        let nodes = BTreeMap::from([(seed.id.clone(), seed)]);
        let inspection_limit = options
            .max_edges
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .context("inspection bound overflow")?;
        Ok(Self {
            nodes,
            edges: BTreeMap::new(),
            parents: BTreeMap::new(),
            queue,
            options,
            target: target.map(str::to_owned),
            found,
            expanded_nodes: 0,
            depth_boundary_nodes: 0,
            inspected_edges: 0,
            inspection_limit,
            detail_bytes,
            truncation: Truncation::default(),
        })
    }

    fn stopped(&self) -> bool {
        self.found || self.truncation.any()
    }

    fn accept(&mut self, current: &str, distance: usize, edge: Relationship) -> Result<()> {
        edge.validate()?;
        self.inspected_edges = self
            .inspected_edges
            .checked_add(1)
            .context("edge count overflow")?;
        if self.inspected_edges > self.inspection_limit {
            self.truncation.inspected_edge_limit = true;
            return Ok(());
        }
        let neighbor = edge.neighbor(current, self.options.direction)?;
        let key = edge.key();
        if self.edges.contains_key(&key) {
            return Ok(());
        }
        if self.edges.len() == self.options.max_edges {
            self.truncation.edge_limit = true;
            return Ok(());
        }
        let candidate = if self.nodes.contains_key(neighbor) {
            None
        } else {
            Some(edge.endpoint(neighbor, distance)?)
        };
        if candidate.is_some() && self.nodes.len() == self.options.max_nodes {
            self.truncation.node_limit = true;
            return Ok(());
        }
        self.insert(current, key, candidate, edge)
    }

    fn insert(
        &mut self,
        current: &str,
        key: EdgeKey,
        candidate: Option<Node>,
        edge: Relationship,
    ) -> Result<()> {
        let node_bytes = candidate
            .as_ref()
            .map(encoded_len)
            .transpose()?
            .unwrap_or_default();
        let edge_bytes = encoded_len(&edge)?;
        let next = self
            .detail_bytes
            .checked_add(node_bytes)
            .and_then(|value| value.checked_add(edge_bytes))
            .context("traversal detail byte count overflow")?;
        anyhow::ensure!(
            edge_bytes <= GRAPH_DETAIL_BYTES,
            "edge metadata exceeds graph bound"
        );
        if next > GRAPH_DETAIL_BYTES {
            self.truncation.detail_byte_limit = true;
            return Ok(());
        }
        if let Some(node) = candidate {
            self.add_node(current, &key, node)?;
        }
        self.detail_bytes = next;
        anyhow::ensure!(
            self.edges.insert(key, edge).is_none(),
            "duplicate traversal edge insertion"
        );
        Ok(())
    }

    fn add_node(&mut self, current: &str, key: &EdgeKey, node: Node) -> Result<()> {
        self.found = self
            .target
            .as_deref()
            .is_some_and(|target| target == node.id);
        let predecessor = Predecessor {
            from: current.into(),
            edge: key.clone(),
        };
        anyhow::ensure!(
            self.parents.insert(node.id.clone(), predecessor).is_none(),
            "duplicate predecessor insertion"
        );
        self.queue.push_back(node.id.clone());
        anyhow::ensure!(
            self.nodes.insert(node.id.clone(), node).is_none(),
            "duplicate node insertion"
        );
        Ok(())
    }

    fn path(&self, start: &str, target: &str) -> Result<Value> {
        if !self.found {
            return Ok(json!({"found":false,"result":"not_found_within_limits",
                "nodes":[],"steps":[],"effective_reachability_inferred":false}));
        }
        let mut cursor = target.to_owned();
        let mut nodes = vec![cursor.clone()];
        let mut steps = Vec::new();
        steps
            .try_reserve(self.options.max_depth)
            .context("reserve path steps")?;
        for _ in 0..=super::MAX_TRAVERSAL_DEPTH {
            if cursor == start {
                break;
            }
            let parent = self
                .parents
                .get(&cursor)
                .context("path predecessor missing")?;
            let edge = self.edges.get(&parent.edge).context("path edge missing")?;
            steps.push(json!({"traversed_from":parent.from,"traversed_to":cursor,
                "reversed":edge.from_id != parent.from,"relationship":edge}));
            cursor.clone_from(&parent.from);
            nodes.push(cursor.clone());
        }
        anyhow::ensure!(cursor == start, "path reconstruction exceeded depth bound");
        nodes.reverse();
        steps.reverse();
        Ok(
            json!({"found":true,"result":"observed_relationship_path","hops":steps.len(),
            "nodes":nodes,"steps":steps,"shortest_for_selected_direction":!self.truncation.any(),
            "effective_reachability_inferred":false}),
        )
    }
}

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
