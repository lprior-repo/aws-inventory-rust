#![forbid(unsafe_code)]
#![deny(unused_must_use)]

mod database;
mod error;
mod graph;

pub use database::{
    Database, DatabaseCounts, DatabaseSnapshot, EdgeCursor, EdgePage, EdgePageRequest, LodSummary,
    MAX_PAGE_LIMIT, PageCursor, PageRequest, ResourceFilter, ResourcePage, ResourceRow,
    ResourceRowPage, ResourceTag, ScopeFreshness, ScopeFreshnessReport, ScopeOutcomeCounts,
    StoredResource, StoredTagObservation, TagFilter,
};
pub use error::ViewError;
pub use graph::{
    GraphConnection, GraphEdge, GraphLimits, GraphNeighborhood, GraphNode, ScopedTag,
    TagAffiliation, TagAffiliations,
};
pub use inventory_core::inventory::{ScopeOutcome, TagCoverage};
