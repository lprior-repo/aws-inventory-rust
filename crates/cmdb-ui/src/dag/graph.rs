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
    Tag,
}

impl NodeKind {
    pub fn from_facet(facet: &str) -> Self {
        let service = facet.split("::").nth(1).map_or("", |value| value);
        match service.to_ascii_lowercase().as_str() {
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
            Self::Tag => 0x60bda5,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DagEdge {
    pub from: usize,
    pub to: usize,
    /// Relationship kind (e.g., "references", "contains"). Preserved for
    /// parallel edges between the same endpoints.
    pub relation: String,
    /// Provenance of the relationship (e.g., "catalog", "reference").
    pub provenance: String,
    pub overlay: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DagGraph {
    nodes: Vec<DagNode>,
    edges: Vec<DagEdge>,
    index: BTreeMap<String, usize>,
    /// Edge identity includes relation and provenance to distinguish parallel
    /// edges between the same endpoints.
    pairs: BTreeSet<(usize, usize, String, String)>,
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
        self.insert_edge(from, to, String::new(), String::new(), false)
    }

    pub fn add_overlay_edge(
        &mut self,
        from: usize,
        to: usize,
        relation: String,
        provenance: String,
    ) -> bool {
        self.insert_edge(from, to, relation, provenance, true)
    }

    /// Adds a hierarchy edge (containment) with an explicit relation name.
    /// Unlike overlay edges, hierarchy edges are used by the layout algorithm
    /// for layer assignment.
    pub fn add_hierarchy_edge(&mut self, from: usize, to: usize, relation: String) -> bool {
        self.insert_edge(from, to, relation, String::new(), false)
    }

    /// Inserts an edge, preserving parallel edges with different relations.
    /// Self-loops (from == to) are allowed and rendered explicitly.
    fn insert_edge(
        &mut self,
        from: usize,
        to: usize,
        relation: String,
        provenance: String,
        overlay: bool,
    ) -> bool {
        if self.nodes.get(from).is_none() || self.nodes.get(to).is_none() {
            return false;
        }
        // Edge identity includes relation and provenance to distinguish
        // parallel edges between the same endpoints (e.g., "references"
        // from catalog vs "references" from reference).
        let edge_key = (from, to, relation.clone(), provenance.clone());
        if self.pairs.contains(&edge_key) {
            return false;
        }
        if self.edges.len() >= MAX_EDGES {
            self.dropped_edges = self.dropped_edges.saturating_add(1);
            return false;
        }
        self.pairs.insert(edge_key);
        self.edges.push(DagEdge {
            from,
            to,
            relation,
            provenance,
            overlay,
        });
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
