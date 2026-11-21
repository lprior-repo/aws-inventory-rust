use super::*;
use std::time::Duration;

const FIELDS: &str = "startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance";

// Each read expands only the admitted frontier, with an early row LIMIT and no
// path enumeration, aggregation or sort. Native graph writes between reads may
// be observed; unlike SQLite's retained transaction this is not a graph snapshot.
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

/// Directed blast-radius: follows only incoming edges to find upstream
/// dependents (what is affected if the root fails). Edge orientation is
/// source->target, so dependents of C are nodes with edges pointing to C.
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

/// Generic graph traversal: expands admitted layers using the provided
/// query builder. For neighborhood queries this is the undirected traversal
/// query; for blast-radius it is the directed incoming-edge query.
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
    for depth in 0u8..=limits.depth {
        if traversal.frontier.is_empty() || traversal.truncated {
            break;
        }
        let fetch = compute_fetch(&traversal, limits, depth)?;
        let query = make_query(&traversal.frontier, &traversal.expanded, fetch);
        let rows = read_layer(&mut connection, graph, &query, fetch).await?;
        if depth == limits.depth {
            traversal.truncated = !rows.is_empty();
            break;
        }
        traversal.add_layer(rows)?;
    }
    Ok(GraphNeighborhood {
        edges: traversal.edges,
        truncated: traversal.truncated,
    })
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

/// Directed blast-radius query: follows incoming edges (n)-[r]->(s) to find
/// upstream dependents (what is affected if this resource fails). The edge
/// orientation is source->target (A->B means A refers to B), so dependents
/// of C are nodes with edges pointing to C.
///
/// Excludes edges to already-expanded nodes but includes edges to frontier
/// nodes, so cycles are captured (e.g., A->C and C->A both appear).
fn blast_radius_query(frontier: &[String], expanded: &[String], limit: usize) -> String {
    let frontier = key_list(frontier);
    let expanded = key_list(expanded);
    format!(
        "UNWIND [{frontier}] AS frontier_key MATCH (n:Resource)-[r]->(s:Resource {{key:frontier_key}}) WHERE NOT n.key IN [{expanded}] RETURN {FIELDS} LIMIT {limit}"
    )
}

fn key_list(keys: &[String]) -> String {
    // Keys originate exclusively from the validated hexadecimal resource digest.
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
        let unseen = usize::from(!self.visited.contains(&from))
            .checked_add(usize::from(from != to && !self.visited.contains(&to)))
            .ok_or(ViewError::GraphLimits)?;
        let nodes = self
            .visited
            .len()
            .checked_add(unseen)
            .ok_or(ViewError::GraphLimits)?;
        if self.edges.len() == self.limits.edges || nodes > self.limits.nodes {
            self.truncated = true;
            return Ok(());
        }
        self.admit_node(from);
        self.admit_node(to);
        self.edges.push(edge);
        Ok(())
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
