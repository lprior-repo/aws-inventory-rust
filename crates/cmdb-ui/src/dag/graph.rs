//! Stable data model for the DAG view.
//!
//! The builder fills this from the filtered fixture, but the model is
//! independent of it: ids are stable strings, duplicate ids collapse to one
//! node, duplicate and self edges are ignored, and both collections are
//! capped so a hostile source cannot grow the layout or the draw pass without
//! bound.

use std::collections::{BTreeMap, BTreeSet};

/// Hard ceiling on nodes retained by [`DagGraph`].
pub const MAX_NODES: usize = 2_000;
/// Hard ceiling on edges retained by [`DagGraph`].
pub const MAX_EDGES: usize = 4_000;
/// Longest label kept verbatim; longer labels get a single ellipsis.
pub const MAX_LABEL_CHARS: usize = 24;

/// What a node represents. Resource kinds carry their own accent colour so a
/// fleet view reads at a glance even before the labels resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    Account,
    Region,
    Kind,
    Ec2,
    S3,
    Rds,
    Lambda,
    Other,
}

impl NodeKind {
    /// Maps the fixture's `kind` facet onto a node kind.
    pub fn from_facet(facet: &str) -> Self {
        match facet {
            "ec2" => Self::Ec2,
            "s3" => Self::S3,
            "rds" => Self::Rds,
            "lambda" => Self::Lambda,
            _ => Self::Other,
        }
    }

    /// Dark-theme accent, packed `0xRRGGBB` for `DrawVector::set_color_hex`.
    pub const fn accent(self) -> u32 {
        match self {
            Self::Account => 0x8f98a8,
            Self::Region => 0x5f8fbf,
            Self::Kind => 0x9b7bd4,
            Self::Ec2 => 0x5aa9e6,
            Self::S3 => 0x55c98a,
            Self::Rds => 0xc08be0,
            Self::Lambda => 0xe0b05a,
            Self::Other => 0x8f98a8,
        }
    }

    /// Resource nodes map to a fixture row; the structural layers do not.
    pub const fn is_resource(self) -> bool {
        matches!(
            self,
            Self::Ec2 | Self::S3 | Self::Rds | Self::Lambda | Self::Other
        )
    }
}

/// One node as the renderer needs it: a stable id, a pre-truncated label and
/// the accent kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DagNode {
    id: String,
    label: String,
    kind: NodeKind,
}

impl DagNode {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }
}

/// A directed edge between two node indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DagEdge {
    pub from: usize,
    pub to: usize,
    /// Overlay edges are drawn but excluded from layering, so same-tier
    /// resource relationships do not stretch the hierarchy.
    pub overlay: bool,
}

/// Node/edge container with deduplication and explicit truncation counters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DagGraph {
    nodes: Vec<DagNode>,
    edges: Vec<DagEdge>,
    index: BTreeMap<String, usize>,
    pairs: BTreeSet<(usize, usize)>,
    dropped_nodes: usize,
    dropped_edges: usize,
}

impl DagGraph {
    /// Adds a node unless the id already exists. Returns the existing index for
    /// a duplicate, `None` once [`MAX_NODES`] is reached.
    pub fn add_node(
        &mut self,
        kind: NodeKind,
        id: impl Into<String>,
        label: impl Into<String>,
    ) -> Option<usize> {
        let id = id.into();
        if let Some(&found) = self.index.get(id.as_str()) {
            return Some(found);
        }
        if self.nodes.len() >= MAX_NODES {
            self.dropped_nodes = self.dropped_nodes.saturating_add(1);
            return None;
        }
        let index = self.nodes.len();
        self.index.insert(id.clone(), index);
        self.nodes.push(DagNode {
            id,
            label: label.into(),
            kind,
        });
        Some(index)
    }

    /// Adds a directed edge. Self loops, duplicates and edges with missing
    /// endpoints are ignored; edges past [`MAX_EDGES`] are counted as dropped.
    pub fn add_edge(&mut self, from: usize, to: usize) -> bool {
        self.insert_edge(from, to, false)
    }

    /// Adds a same-tier data-flow edge; layout ignores it for layering.
    pub fn add_overlay_edge(&mut self, from: usize, to: usize) -> bool {
        self.insert_edge(from, to, true)
    }

    fn insert_edge(&mut self, from: usize, to: usize, overlay: bool) -> bool {
        if from == to || self.nodes.get(from).is_none() || self.nodes.get(to).is_none() {
            return false;
        }
        if self.pairs.contains(&(from, to)) {
            return false;
        }
        if self.edges.len() >= MAX_EDGES {
            self.dropped_edges = self.dropped_edges.saturating_add(1);
            return false;
        }
        self.pairs.insert((from, to));
        self.edges.push(DagEdge { from, to, overlay });
        true
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    pub fn node(&self, index: usize) -> Option<&DagNode> {
        self.nodes.get(index)
    }

    pub fn nodes(&self) -> &[DagNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[DagEdge] {
        &self.edges
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn dropped_nodes(&self) -> usize {
        self.dropped_nodes
    }

    pub fn dropped_edges(&self) -> usize {
        self.dropped_edges
    }
}

/// Caps a label at [`MAX_LABEL_CHARS`] characters, ending with one ellipsis.
pub fn truncate_label(text: &str) -> String {
    if text.chars().count() <= MAX_LABEL_CHARS {
        return text.to_owned();
    }
    let keep = MAX_LABEL_CHARS.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    out
}
