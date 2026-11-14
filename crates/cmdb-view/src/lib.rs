//! Bounded, deterministic read model over `aws-inventory export --format json`.
//!
//! The crate parses one published snapshot into sorted resource views, a
//! prebuilt edge adjacency index and LOD counts, then answers filtered pages and
//! depth-limited graph neighborhoods without touching the source bytes again.
//! It performs no network, database, filesystem or clock I/O.
//!
//! # Parsed contract
//!
//! The document is the versioned JSON export:
//!
//! - `header.artifact`, `header.version`, `header.provenance`,
//!   `header.scan.{scan_id,status,started_at,finished_at,resources,edges,
//!   completed_scopes,failed_scopes}`, plus optional `header.catalog_digest` and
//!   `header.evidence_digest`.
//! - `records[]` tagged `{ "record": tag, "data": payload }`:
//!   `resource.data.resource.{id,account_id,region,kind,name,properties,tags}`,
//!   `edge.data.edge.{from_id,to_id,relation}`,
//!   `scope.data.{scope.{account_id,region,collector,parent_id},complete,failed}`,
//!   `failure.data.{scope.*,category,message}`.
//!
//! Unknown fields are ignored; unknown record tags, missing identities and
//! unsupported artifacts/versions are typed errors. All-observations exports
//! repeat identities; the first input record for each resource ID wins, even
//! when its account, region or kind differs from a later observation. Resources
//! are then displayed in `(account, region, kind, id)` order; lookup uses ID alone.
//!
//! # Bounds
//!
//! Input is capped at 1 GiB, resources at 2,000,000 and edges at 5,000,000
//! records (exceeding them is an error). Scopes and failures are retained up to
//! 500,000 each and flagged truncated beyond that. Pages, graph neighborhoods
//! and LOD summaries report their own caps.

#![deny(missing_docs)]
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
