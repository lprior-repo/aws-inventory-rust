//! Directed edge projection, graph limits and bounded neighborhood results.

use crate::Snapshot;
use crate::error::ViewError;
use crate::resource::ResourceView;

mod traverse;

pub(crate) use traverse::build_adjacency;

/// Deepest neighborhood the UI may request.
pub const MAX_GRAPH_DEPTH: u8 = 2;
/// Largest node set one neighborhood may return.
pub const MAX_GRAPH_NODES: usize = 10_000;
/// Largest edge set one neighborhood may return.
pub const MAX_GRAPH_EDGES: usize = 20_000;

/// The UI-facing projection of one `edge` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeView {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
}

impl EdgeView {
    /// Stable ordering key `(from_id, to_id, relation)`.
    pub fn sort_key(&self) -> (&str, &str, &str) {
        (&self.from_id, &self.to_id, &self.relation)
    }
}

/// Bounded traversal request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    depth: u8,
    max_nodes: usize,
    max_edges: usize,
}

impl GraphLimits {
    /// Validate depth (`1..=2`) and caps (`1..=MAX_GRAPH_*`).
    pub fn new(depth: u8, max_nodes: usize, max_edges: usize) -> Result<Self, ViewError> {
        if !(1..=MAX_GRAPH_DEPTH).contains(&depth) {
            return Err(ViewError::InvalidGraphDepth);
        }
        if !(1..=MAX_GRAPH_NODES).contains(&max_nodes)
            || !(1..=MAX_GRAPH_EDGES).contains(&max_edges)
        {
            return Err(ViewError::InvalidGraphLimit);
        }
        Ok(Self {
            depth,
            max_nodes,
            max_edges,
        })
    }

    /// Requested traversal depth.
    pub const fn depth(self) -> u8 {
        self.depth
    }

    /// Node cap including the root.
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// Edge cap.
    pub const fn max_edges(self) -> usize {
        self.max_edges
    }
}

/// One node of a returned neighborhood.
///
/// `resource` is `None` when the id is referenced by an edge but absent from
/// the snapshot's resource records (a dangling target). Such nodes are marked,
/// never dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphNode<'a> {
    pub id: &'a str,
    pub resource: Option<&'a ResourceView>,
}

impl GraphNode<'_> {
    /// Whether the referenced resource exists in this snapshot.
    pub const fn is_present(&self) -> bool {
        self.resource.is_some()
    }
}

/// A bounded, deterministically ordered subgraph.
#[derive(Clone, Debug)]
pub struct Subgraph<'a> {
    pub root: GraphNode<'a>,
    /// Included nodes, sorted by id; includes the root and dangling targets.
    pub nodes: Vec<GraphNode<'a>>,
    /// Included edges, sorted by `(from_id, to_id, relation)`.
    pub edges: Vec<&'a EdgeView>,
    /// The node cap prevented including every reachable node.
    pub truncated_nodes: bool,
    /// The edge cap prevented including every eligible edge.
    pub truncated_edges: bool,
    /// Edges from boundary nodes lead beyond the requested depth.
    pub truncated_depth: bool,
}

impl Snapshot {
    /// Sorted edge projection shared by LOD and graph queries.
    pub fn edges(&self) -> &[EdgeView] {
        &self.edges
    }

    /// Bounded neighborhood around `root_id`.
    ///
    /// An unknown root is represented as a single absent node rather than an
    /// error, so a UI can inspect dangling edge targets.
    pub fn neighborhood<'a>(&'a self, root_id: &'a str, limits: GraphLimits) -> Subgraph<'a> {
        traverse::neighborhood(
            root_id,
            limits,
            &self.resources,
            &self.resource_index,
            &self.edges,
            &self.adjacency,
        )
    }
}
