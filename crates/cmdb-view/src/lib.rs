#![forbid(unsafe_code)]
#![deny(unused_must_use)]

mod build;
mod error;
mod graph;
mod lod;
mod meta;
mod parse;
mod raw;
mod resource;

pub use error::ViewError;
pub use graph::{
    EdgeView, GraphLimits, GraphNode, MAX_GRAPH_DEPTH, MAX_GRAPH_EDGES, MAX_GRAPH_NODES, Subgraph,
};
pub use lod::{CountBucket, DegreeEntry, LodLimits, LodSummary, MAX_LOD_BUCKETS, MAX_TOP_DEGREE};
pub use meta::{
    Coverage, FailureView, Provenance, SNAPSHOT_ARTIFACT, SUPPORTED_VERSION, ScanStatus,
    ScopeCoverage, SnapshotMeta,
};
pub use parse::Snapshot;
pub use resource::{
    MAX_PAGE_LIMIT, PageCursor, PageRequest, ResourceFilter, ResourcePage, ResourceView,
};
