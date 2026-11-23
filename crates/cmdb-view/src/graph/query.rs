use super::*;
use std::time::Duration;

const FIELDS: &str = "startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance";

pub(super) async fn neighborhood(
    client: &redis::Client,
    graph: &str,
    root: &GraphNode,
    limits: GraphLimits,
) -> Result<GraphNeighborhood, ViewError> {
    tokio::time::timeout(
        Duration::from_secs(10),
        traverse_with_query(client, graph, root, limits, traversal_query),
    )
    .await
    .map_err(|_| ViewError::GraphTimeout)?
}

pub(super) async fn blast_radius(
    client: &redis::Client,
    graph: &str,
    root: &GraphNode,
    limits: GraphLimits,
) -> Result<GraphNeighborhood, ViewError> {
    tokio::time::timeout(
        Duration::from_secs(10),
        traverse_with_query(client, graph, root, limits, blast_radius_query),
    )
    .await
    .map_err(|_| ViewError::GraphTimeout)?
}

async fn traverse_with_query<F>(
    client: &redis::Client,
    graph: &str,
    root: &GraphNode,
    limits: GraphLimits,
    make_query: F,
) -> Result<GraphNeighborhood, ViewError>
where
    F: Fn(&[String], &[String], usize) -> String,
{
    let mut traversal = Traversal::new(key_digest(root)?, limits);
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(ViewError::Graph)?;
    expand_layers(&mut traversal, &mut connection, graph, limits, make_query).await?;
    Ok(GraphNeighborhood {
        edges: traversal.edges,
        truncated: traversal.truncated,
    })
}

async fn expand_layers<F>(
    traversal: &mut Traversal,
    connection: &mut redis::aio::MultiplexedConnection,
    graph: &str,
    limits: GraphLimits,
    make_query: F,
) -> Result<(), ViewError>
where
    F: Fn(&[String], &[String], usize) -> String,
{
    for depth in 0u8..=limits.depth {
        if traversal.frontier.is_empty() || traversal.truncated {
            break;
        }
        let fetch = compute_fetch(traversal, limits, depth)?;
        let query = make_query(&traversal.frontier, &traversal.expanded, fetch);
        let rows = read_layer(connection, graph, &query, fetch).await?;
        if depth == limits.depth {
            traversal.truncated = !rows.is_empty();
            break;
        }
        traversal.add_layer(rows)?;
    }
    Ok(())
}

fn compute_fetch(
    traversal: &Traversal,
    limits: GraphLimits,
    depth: u8,
) -> Result<usize, ViewError> {
    let remaining = limits
        .edges
        .checked_sub(traversal.edges.len())
        .ok_or(ViewError::GraphLimits)?;
    if depth == limits.depth {
        Ok(1)
    } else {
        remaining.checked_add(1).ok_or(ViewError::GraphLimits)
    }
}

async fn read_layer(
    connection: &mut redis::aio::MultiplexedConnection,
    graph: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<GraphEdge>, ViewError> {
    let value = redis::cmd("GRAPH.RO_QUERY")
        .arg(graph)
        .arg(query)
        .arg("TIMEOUT")
        .arg(10_000_u32)
        .query_async::<redis::Value>(connection)
        .await
        .map_err(ViewError::Graph)?;
    parse_rows(value, limit)
}

fn traversal_query(frontier: &[String], expanded: &[String], limit: usize) -> String {
    let frontier = key_list(frontier);
    let expanded = key_list(expanded);
    format!(
        "UNWIND [{frontier}] AS frontier_key MATCH (s:Resource {{key:frontier_key}})-[r]-(n:Resource) WHERE NOT n.key IN [{expanded}] AND (NOT n.key IN [{frontier}] OR startNode(r)=s) RETURN {FIELDS} LIMIT {limit}"
    )
}

fn blast_radius_query(frontier: &[String], expanded: &[String], limit: usize) -> String {
    let frontier = key_list(frontier);
    let expanded = key_list(expanded);
    format!(
        "UNWIND [{frontier}] AS frontier_key MATCH (n:Resource)-[r]->(s:Resource {{key:frontier_key}}) WHERE NOT n.key IN [{expanded}] RETURN {FIELDS} LIMIT {limit}"
    )
}

fn key_list(keys: &[String]) -> String {
    keys.iter().fold(
        String::with_capacity(keys.len().saturating_mul(67)),
        |mut list, key| {
            if !list.is_empty() {
                list.push(',');
            }
            list.push('\'');
            list.push_str(key);
            list.push('\'');
            list
        },
    )
}

fn parse_rows(value: redis::Value, limit: usize) -> Result<Vec<GraphEdge>, ViewError> {
    let redis::Value::Array(parts) = value else {
        return Err(ViewError::GraphResponse);
    };
    let Some(redis::Value::Array(rows)) = parts.get(1) else {
        return Err(ViewError::GraphResponse);
    };
    if rows.len() > limit {
        return Err(ViewError::GraphResponse);
    }
    rows.iter()
        .map(|row| {
            let redis::Value::Array(fields) = row else {
                return Err(ViewError::GraphResponse);
            };
            if fields.len() != 10 {
                return Err(ViewError::GraphResponse);
            }
            let edge = response::parse_edge(row)?;
            key_digest(&edge.from).map_err(|_| ViewError::GraphResponse)?;
            key_digest(&edge.to).map_err(|_| ViewError::GraphResponse)?;
            Ok(edge)
        })
        .collect()
}

struct Traversal {
    frontier: Vec<String>,
    expanded: Vec<String>,
    visited: Vec<String>,
    edges: Vec<GraphEdge>,
    limits: GraphLimits,
    truncated: bool,
}

impl Traversal {
    fn new(root: String, limits: GraphLimits) -> Self {
        Self {
            frontier: vec![root.clone()],
            expanded: Vec::new(),
            visited: vec![root],
            edges: Vec::new(),
            limits,
            truncated: false,
        }
    }

    fn add_layer(&mut self, rows: Vec<GraphEdge>) -> Result<(), ViewError> {
        let current = std::mem::take(&mut self.frontier);
        self.expanded.extend(current.iter().cloned());
        rows.into_iter().try_for_each(|edge| {
            if !self.truncated {
                self.add_edge(edge, &current)?;
            }
            Ok(())
        })
    }

    fn add_edge(&mut self, edge: GraphEdge, current: &[String]) -> Result<(), ViewError> {
        let from = key_digest(&edge.from)?;
        let to = key_digest(&edge.to)?;
        if !current.contains(&from) && !current.contains(&to) {
            return Err(ViewError::GraphResponse);
        }
        if self.edges.contains(&edge) {
            return Ok(());
        }
        if self.exceeds_budget(&from, &to)? {
            self.truncated = true;
            return Ok(());
        }
        self.admit_node(from);
        self.admit_node(to);
        self.edges.push(edge);
        Ok(())
    }

    fn exceeds_budget(&self, from: &str, to: &str) -> Result<bool, ViewError> {
        let unseen = usize::from(!self.visited.iter().any(|key| key == from))
            .checked_add(usize::from(
                from != to && !self.visited.iter().any(|key| key == to),
            ))
            .ok_or(ViewError::GraphLimits)?;
        let nodes = self
            .visited
            .len()
            .checked_add(unseen)
            .ok_or(ViewError::GraphLimits)?;
        Ok(self.edges.len() == self.limits.edges || nodes > self.limits.nodes)
    }

    fn admit_node(&mut self, key: String) {
        if !self.visited.contains(&key) {
            self.visited.push(key.clone());
            self.frontier.push(key);
        }
    }
}

#[cfg(test)]
#[path = "query/tests.rs"]
mod tests;
