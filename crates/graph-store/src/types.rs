use crate::{GraphError, MAX_DEPTH, MAX_EDGES, MAX_NODES, NodeKey};
use inventory_core::inventory::Relation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeRow {
    pub key: NodeKey,
    pub name: Option<String>,
    pub content_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Provenance(String);

impl Provenance {
    pub fn parse(value: &str) -> Result<Self, GraphError> {
        if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
            return Err(GraphError::InvalidRow);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeRow {
    pub from: NodeKey,
    pub to: NodeKey,
    pub relation: Relation,
    pub provenance: Provenance,
}

pub type EdgeKey = EdgeRow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    depth: u8,
    nodes: usize,
    edges: usize,
}

impl GraphLimits {
    pub fn new(depth: u8, nodes: usize, edges: usize) -> Result<Self, GraphError> {
        if depth == 0
            || depth > MAX_DEPTH
            || nodes == 0
            || nodes > MAX_NODES
            || edges == 0
            || edges > MAX_EDGES
        {
            return Err(GraphError::Limits);
        }
        Ok(Self {
            depth,
            nodes,
            edges,
        })
    }

    pub fn depth(self) -> u8 {
        self.depth
    }

    pub fn nodes(self) -> usize {
        self.nodes
    }

    pub fn edges(self) -> usize {
        self.edges
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNeighborhood {
    pub edges: Vec<EdgeRow>,
    pub truncation: Truncation,
}

impl GraphNeighborhood {
    pub fn truncated(&self) -> bool {
        self.truncation != Truncation::Complete
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truncation {
    Complete,
    Depth,
    Budget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathResult {
    NotFound,
    SameNode,
    Path(Vec<EdgeRow>),
}
