mod path;

use crate::cypher::Direction;
use crate::falkor::FalkorDb;
use crate::{
    EdgeRow, GraphError, GraphLimits, GraphNeighborhood, NodeKey, PathResult, Truncation, cypher,
    response,
};

pub(crate) async fn read(
    graph: &FalkorDb,
    root: &NodeKey,
    limits: GraphLimits,
    direction: Direction,
) -> Result<GraphNeighborhood, GraphError> {
    let mut state = Traversal::new(root, limits)?;
    expand(graph, &mut state, direction, 0).await?;
    expand(graph, &mut state, direction, 1).await?;
    finish(graph, &mut state, direction).await?;
    Ok(GraphNeighborhood {
        edges: state.edges,
        truncation: state.truncation,
    })
}

async fn finish(
    graph: &FalkorDb,
    state: &mut Traversal,
    direction: Direction,
) -> Result<(), GraphError> {
    expand(graph, state, direction, 2).await?;
    expand(graph, state, direction, 3).await
}

async fn expand(
    graph: &FalkorDb,
    state: &mut Traversal,
    direction: Direction,
    depth: u8,
) -> Result<(), GraphError> {
    if state.frontier.is_empty()
        || state.truncation != Truncation::Complete
        || depth > state.limits.depth()
    {
        return Ok(());
    }
    let rows = load_layer(graph, state, direction, depth).await?;
    if depth == state.limits.depth() {
        if !rows.is_empty() {
            state.truncation = Truncation::Depth;
        }
        return Ok(());
    }
    state.add_layer(rows)
}

async fn load_layer(
    graph: &FalkorDb,
    state: &Traversal,
    direction: Direction,
    depth: u8,
) -> Result<Vec<EdgeRow>, GraphError> {
    let remaining = state
        .limits
        .edges()
        .checked_sub(state.edges.len())
        .ok_or(GraphError::Limits)?;
    let fetch = if depth == state.limits.depth() {
        1
    } else {
        remaining.saturating_add(1)
    };
    let query = cypher::layer(&state.frontier, &state.expanded, fetch, direction)?;
    let value = graph.request(&query).await?;
    response::edges(&value, fetch)
}

pub(crate) fn shortest(
    neighborhood: &GraphNeighborhood,
    from: &NodeKey,
    to: &NodeKey,
    limits: GraphLimits,
) -> Result<PathResult, GraphError> {
    match path::shortest(&neighborhood.edges, from, to, limits)? {
        Some(edges) => Ok(PathResult::Path(edges)),
        None if neighborhood.truncation == Truncation::Budget => Err(GraphError::Truncated),
        None => Ok(PathResult::NotFound),
    }
}

struct Traversal {
    frontier: Vec<NodeKey>,
    expanded: Vec<NodeKey>,
    visited: Vec<NodeKey>,
    edges: Vec<EdgeRow>,
    limits: GraphLimits,
    truncation: Truncation,
}

impl Traversal {
    fn new(root: &NodeKey, limits: GraphLimits) -> Result<Self, GraphError> {
        let mut frontier = buffer(limits.nodes())?;
        let mut visited = buffer(limits.nodes())?;
        frontier.push(root.clone());
        visited.push(root.clone());
        Ok(Self {
            frontier,
            visited,
            expanded: buffer(limits.nodes())?,
            edges: buffer(limits.edges())?,
            limits,
            truncation: Truncation::Complete,
        })
    }

    fn add_layer(&mut self, rows: Vec<EdgeRow>) -> Result<(), GraphError> {
        let current = std::mem::replace(&mut self.frontier, buffer(self.limits.nodes())?);
        self.expanded.extend(current.iter().cloned());
        rows.into_iter().try_for_each(|edge| {
            if self.truncation == Truncation::Complete {
                self.add_edge(edge, &current)?;
            }
            Ok(())
        })
    }

    fn add_edge(&mut self, edge: EdgeRow, current: &[NodeKey]) -> Result<(), GraphError> {
        if !current.contains(&edge.from) && !current.contains(&edge.to) {
            return Err(GraphError::Response);
        }
        if self.edges.contains(&edge) {
            return Ok(());
        }
        let unseen = usize::from(!self.visited.contains(&edge.from)).saturating_add(usize::from(
            edge.from != edge.to && !self.visited.contains(&edge.to),
        ));
        if self.edges.len() == self.limits.edges()
            || self.visited.len().saturating_add(unseen) > self.limits.nodes()
        {
            self.truncation = Truncation::Budget;
            return Ok(());
        }
        self.admit(&edge.from);
        self.admit(&edge.to);
        self.edges.push(edge);
        Ok(())
    }

    fn admit(&mut self, key: &NodeKey) {
        if !self.visited.contains(key) {
            self.visited.push(key.clone());
            self.frontier.push(key.clone());
        }
    }
}

pub(crate) fn buffer<T>(capacity: usize) -> Result<Vec<T>, GraphError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| GraphError::Allocation)?;
    Ok(values)
}

#[cfg(test)]
mod tests;
