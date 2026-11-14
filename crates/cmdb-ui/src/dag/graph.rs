use std::collections::{BTreeMap, BTreeSet};

pub const MAX_NODES: usize = 2_000;

pub const MAX_EDGES: usize = 4_000;

pub const MAX_LABEL_CHARS: usize = 24;

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
    pub fn from_facet(facet: &str) -> Self {
        match facet {
            "ec2" => Self::Ec2,
            "s3" => Self::S3,
            "rds" => Self::Rds,
            "lambda" => Self::Lambda,
            _ => Self::Other,
        }
    }

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

    pub const fn is_resource(self) -> bool {
        matches!(
            self,
            Self::Ec2 | Self::S3 | Self::Rds | Self::Lambda | Self::Other
        )
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DagEdge {
    pub from: usize,
    pub to: usize,

    pub overlay: bool,
}

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

    pub fn add_edge(&mut self, from: usize, to: usize) -> bool {
        self.insert_edge(from, to, false)
    }

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

pub fn truncate_label(text: &str) -> String {
    if text.chars().count() <= MAX_LABEL_CHARS {
        return text.to_owned();
    }
    let keep = MAX_LABEL_CHARS.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    out
}
