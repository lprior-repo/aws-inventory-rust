use inventory_core::inventory::{Relation, ResourceKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionState {
    pub pending: u64,
    pub rebuild_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionResource {
    pub key: ResourceKey,
    pub name: Option<String>,
    pub content_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionEdgeHeader {
    pub from: ResourceKey,
    pub to: ResourceKey,
    pub relation: Relation,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionEdgeCursor {
    pub(in crate::inventory) from_account: String,
    pub(in crate::inventory) from_region: String,
    pub(in crate::inventory) from_kind: String,
    pub(in crate::inventory) from_id: String,
    pub(in crate::inventory) to_account: String,
    pub(in crate::inventory) to_region: String,
    pub(in crate::inventory) to_kind: String,
    pub(in crate::inventory) to_id: String,
    pub(in crate::inventory) relation: String,
    pub(in crate::inventory) source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionEdgePage {
    pub edges: Vec<ProjectionEdgeHeader>,
    pub next: Option<ProjectionEdgeCursor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTag {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTagCursor(pub(in crate::inventory) String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTagPage {
    pub tags: Vec<ProjectionTag>,
    pub next: Option<ProjectionTagCursor>,
}
