#[path = "graph/path.rs"]
mod path;
use crate::ViewError;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use inventory_graph::{GraphBackend, GraphError, falkor::FalkorDb};
pub use inventory_graph::{ScopedTag, TagAffiliation, TagAffiliations};

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

/// Outcome of a shortest-path search, including searches cut short by a bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathResult {
    /// No path exists between the source and target within the depth limit.
    NotFound,
    /// A search bound prevented establishing whether a path exists.
    Truncated,
    /// Source and target are the same node; the path has zero hops.
    SameNode,
    /// A path was found. Edges are returned in traversal order.
    Path(Vec<GraphEdge>),
}

pub struct GraphConnection<G = FalkorDb> {
    backend: G,
}

impl GraphConnection<FalkorDb> {
    pub fn connect(endpoint: &str, graph: &str) -> Result<Self, ViewError> {
        Ok(Self::from_backend(
            FalkorDb::connect(endpoint, graph).map_err(graph_error)?,
        ))
    }
}

impl<G: GraphBackend> GraphConnection<G> {
    pub fn from_backend(backend: G) -> Self {
        Self { backend }
    }
    /// Live native tag membership, separate from AWS dependency traversal.
    /// This does not share SQLite's retained snapshot or pin a native revision.
    pub async fn tag_affiliations(
        &self,
        node: &GraphNode,
        limit: usize,
    ) -> Result<TagAffiliations, ViewError> {
        if limit == 0 || limit > crate::MAX_PAGE_LIMIT {
            return Err(ViewError::PageLimit);
        }
        self.backend
            .tag_affiliations(
                &node_key(node)?,
                u16::try_from(limit).map_err(|_| ViewError::PageLimit)?,
            )
            .await
            .map_err(graph_error)
    }

    pub async fn neighbors(
        &self,
        node: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        let limits = backend_limits(limits)?;
        let result = self
            .backend
            .neighbors(&node_key(node)?, limits)
            .await
            .map_err(graph_error)?;
        Ok(neighborhood(result))
    }

    /// Returns upstream dependents only (what is affected if this node
    /// fails), following incoming edges. Edge orientation is source->target
    /// (A->B means A refers to B), so dependents of C are nodes that refer
    /// to C. This differs from `neighbors` which traverses in both
    /// directions. A node with no incoming edges has empty blast radius.
    pub async fn blast_radius(
        &self,
        node: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        let limits = backend_limits(limits)?;
        let result = self
            .backend
            .blast_radius(&node_key(node)?, limits)
            .await
            .map_err(graph_error)?;
        Ok(neighborhood(result))
    }

    /// Finds the shortest path between two nodes. Returns a PathResult that
    /// distinguishes NotFound (no path within depth limit), SameNode (source
    /// and target are the same node with zero hops), and Path(edges) when a
    /// path is found.
    pub async fn path(
        &self,
        from: &GraphNode,
        to: &GraphNode,
        limits: GraphLimits,
    ) -> Result<PathResult, ViewError> {
        let limits = backend_limits(limits)?;
        let result = self
            .backend
            .path(&node_key(from)?, &node_key(to)?, limits)
            .await;
        path::convert(result)
    }
}

fn backend_limits(limits: GraphLimits) -> Result<inventory_graph::GraphLimits, ViewError> {
    inventory_graph::GraphLimits::new(limits.depth, limits.nodes, limits.edges).map_err(graph_error)
}

fn node_key(node: &GraphNode) -> Result<ResourceKey, ViewError> {
    Ok(ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    })
}

fn graph_error(error: GraphError) -> ViewError {
    match error {
        GraphError::Timeout => ViewError::GraphTimeout,
        GraphError::GraphName => ViewError::GraphName,
        GraphError::Limits => ViewError::GraphLimits,
        GraphError::Response | GraphError::InvalidRow => ViewError::GraphResponse,
        error => ViewError::Backend(error),
    }
}

fn neighborhood(result: inventory_graph::GraphNeighborhood) -> GraphNeighborhood {
    let truncated = result.truncated();
    GraphNeighborhood {
        edges: result.edges.into_iter().map(graph_edge).collect(),
        truncated,
    }
}

fn graph_node(key: ResourceKey) -> GraphNode {
    GraphNode {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
    }
}

fn graph_edge(edge: inventory_graph::EdgeRow) -> GraphEdge {
    GraphEdge {
        from: graph_node(edge.from),
        to: graph_node(edge.to),
        relation: edge.relation.as_str().to_owned(),
        provenance: edge.provenance.as_str().to_owned(),
    }
}

#[cfg(test)]
#[path = "graph/view_tests.rs"]
mod view_tests;
