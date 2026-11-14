use crate::Snapshot;
use crate::error::ViewError;
use crate::resource::ResourceView;

mod traverse;

pub(crate) use traverse::build_adjacency;

pub const MAX_GRAPH_DEPTH: u8 = 2;

pub const MAX_GRAPH_NODES: usize = 10_000;

pub const MAX_GRAPH_EDGES: usize = 20_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeView {
    pub from_id: String,

    pub to_id: String,

    pub relation: String,
}

impl EdgeView {
    pub fn sort_key(&self) -> (&str, &str, &str) {
        (&self.from_id, &self.to_id, &self.relation)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    depth: u8,
    max_nodes: usize,
    max_edges: usize,
}

impl GraphLimits {
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

    pub const fn depth(self) -> u8 {
        self.depth
    }

    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    pub const fn max_edges(self) -> usize {
        self.max_edges
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphNode<'a> {
    pub id: &'a str,

    pub resource: Option<&'a ResourceView>,
}

impl GraphNode<'_> {
    pub const fn is_present(&self) -> bool {
        self.resource.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct Subgraph<'a> {
    pub root: GraphNode<'a>,

    pub nodes: Vec<GraphNode<'a>>,

    pub edges: Vec<&'a EdgeView>,

    pub truncated_nodes: bool,

    pub truncated_edges: bool,

    pub truncated_depth: bool,
}

impl Snapshot {
    pub fn edges(&self) -> &[EdgeView] {
        &self.edges
    }

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
