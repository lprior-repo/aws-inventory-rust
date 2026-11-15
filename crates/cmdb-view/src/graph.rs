#[path = "graph/response.rs"]
mod response;
use crate::ViewError;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use response::*;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNode {
    pub account: String,
    pub region: String,
    pub kind: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: GraphNode,
    pub to: GraphNode,
    pub relation: String,
    pub provenance: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNeighborhood {
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    pub depth: u8,
    pub nodes: usize,
    pub edges: usize,
}

pub struct GraphConnection {
    client: redis::Client,
    graph: String,
}

impl GraphConnection {
    pub fn connect(endpoint: &str, graph: &str) -> Result<Self, ViewError> {
        if graph.is_empty() || graph.chars().any(char::is_control) {
            return Err(ViewError::GraphName);
        }
        let client = redis::Client::open(endpoint).map_err(ViewError::Graph)?;
        Ok(Self {
            client,
            graph: graph.to_owned(),
        })
    }

    pub async fn neighbors(
        &self,
        node: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        validate_limits(limits)?;
        let mut connection = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.get_multiplexed_async_connection(),
        )
        .await
        .map_err(|_| ViewError::GraphTimeout)?
        .map_err(ViewError::Graph)?;
        let mut traversal = Traversal::new(key_digest(node)?, limits);
        self.expand(&mut connection, &mut traversal).await?;
        Ok(GraphNeighborhood {
            edges: traversal.edges,
            truncated: traversal.truncated,
        })
    }

    async fn expand(
        &self,
        connection: &mut redis::aio::MultiplexedConnection,
        traversal: &mut Traversal,
    ) -> Result<(), ViewError> {
        for _ in 0..traversal.limits.depth {
            let remaining = traversal.limits.edges.saturating_sub(traversal.edges.len());
            let fetch_limit = remaining.saturating_add(1);
            let query = traversal_query(&traversal.frontier, &[], fetch_limit);
            let layer = query_layer(connection, &self.graph, &query, fetch_limit).await?;
            traversal.add_layer(layer, remaining)?;
            if traversal.frontier.is_empty() || traversal.truncated {
                return Ok(());
            }
        }
        let query = traversal_query(&traversal.frontier, &traversal.visited, 1);
        traversal.truncated = !query_layer(connection, &self.graph, &query, 1)
            .await?
            .is_empty();
        Ok(())
    }
}

fn validate_limits(limits: GraphLimits) -> Result<(), ViewError> {
    if limits.depth == 0
        || limits.depth > 3
        || limits.edges == 0
        || limits.edges > 2000
        || limits.nodes == 0
        || limits.nodes > 500
    {
        return Err(ViewError::GraphLimits);
    }
    Ok(())
}

async fn query_layer(
    connection: &mut redis::aio::MultiplexedConnection,
    graph: &str,
    query: &str,
    edge_limit: usize,
) -> Result<Vec<GraphEdge>, ViewError> {
    let mut command = redis::cmd("GRAPH.QUERY");
    command.arg(graph).arg(query);
    let operation = command.query_async(connection);
    let response: redis::Value = tokio::time::timeout(Duration::from_secs(10), operation)
        .await
        .map_err(|_| ViewError::GraphTimeout)?
        .map_err(ViewError::Graph)?;
    parse_edges(response, edge_limit)
}

fn traversal_query(keys: &[String], excluded: &[String], edge_limit: usize) -> String {
    let size = keys
        .len()
        .saturating_add(excluded.len())
        .saturating_mul(35)
        .saturating_add(500);
    let mut query = String::with_capacity(size);
    query.push_str("MATCH (s:Resource)-[r]-(n:Resource) WHERE s.key IN [");
    push_key_list(&mut query, keys);
    query.push(']');
    if !excluded.is_empty() {
        query.push_str(" AND n.key NOT IN [");
        push_key_list(&mut query, excluded);
        query.push(']');
    }
    query.push_str(" RETURN DISTINCT startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance ORDER BY startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance LIMIT ");
    query.push_str(&edge_limit.to_string());
    query
}

fn push_key_list(query: &mut String, keys: &[String]) {
    for (position, key) in keys.iter().enumerate() {
        if position > 0 {
            query.push(',');
        }
        query.push('\'');
        query.push_str(key);
        query.push('\'');
    }
}

fn key_digest(node: &GraphNode) -> Result<String, ViewError> {
    let key = ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    };
    let mut digest = String::with_capacity(64);
    for byte in key.digest().as_bytes() {
        std::fmt::Write::write_fmt(&mut digest, format_args!("{byte:02x}"))
            .map_err(|_| ViewError::GraphResponse)?;
    }
    Ok(digest)
}

fn parse_edges(response: redis::Value, max_edges: usize) -> Result<Vec<GraphEdge>, ViewError> {
    let redis::Value::Array(parts) = response else {
        return Err(ViewError::GraphResponse);
    };
    let Some(redis::Value::Array(rows)) = parts.get(1) else {
        return Err(ViewError::GraphResponse);
    };
    if rows.len() > max_edges {
        return Err(ViewError::GraphLimits);
    }
    let mut edges = Vec::with_capacity(rows.len());
    for row in rows {
        edges.push(parse_edge(row)?);
    }
    Ok(edges)
}

struct Traversal {
    frontier: Vec<String>,
    visited: Vec<String>,
    edges: Vec<GraphEdge>,
    limits: GraphLimits,
    truncated: bool,
}

impl Traversal {
    fn new(root: String, limits: GraphLimits) -> Self {
        Self {
            frontier: vec![root.clone()],
            visited: vec![root],
            edges: Vec::new(),
            limits,
            truncated: false,
        }
    }

    fn add_layer(&mut self, layer: Vec<GraphEdge>, edge_budget: usize) -> Result<(), ViewError> {
        let current = std::mem::take(&mut self.frontier);
        let mut added = 0;
        for edge in layer {
            if self.edges.contains(&edge) {
                continue;
            }
            if added >= edge_budget {
                self.truncated = true;
                break;
            }
            let before = self.edges.len();
            self.add_edge(edge, &current)?;
            if self.edges.len() > before {
                added = added.saturating_add(1);
            }
            if self.truncated {
                break;
            }
        }
        Ok(())
    }

    fn add_edge(&mut self, edge: GraphEdge, current: &[String]) -> Result<(), ViewError> {
        if self.edges.contains(&edge) {
            return Ok(());
        }
        if self.edges.len() >= self.limits.edges {
            self.truncated = true;
            return Ok(());
        }
        let from = key_digest(&edge.from)?;
        let to = key_digest(&edge.to)?;
        if !current.contains(&from) && !current.contains(&to) {
            return Ok(());
        }
        let neighbor = if current.contains(&from) {
            edge.to.clone()
        } else {
            edge.from.clone()
        };
        if !self.add_neighbor(neighbor)? {
            self.truncated = true;
            return Ok(());
        }
        self.edges.push(edge);
        Ok(())
    }

    fn add_neighbor(&mut self, node: GraphNode) -> Result<bool, ViewError> {
        let key = key_digest(&node)?;
        if self.visited.contains(&key) {
            return Ok(true);
        }
        if self.visited.len() >= self.limits.nodes {
            return Ok(false);
        }
        self.visited.push(key.clone());
        self.frontier.push(key);
        Ok(true)
    }
}
