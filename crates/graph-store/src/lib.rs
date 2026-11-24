mod encoding;
mod fence;
mod response;
mod tags;
mod traversal;
mod types;

pub mod cypher;
pub mod falkor;

pub use inventory_core::inventory::ResourceKey as NodeKey;
pub use types::{
    EdgeKey, EdgeRow, GraphLimits, GraphNeighborhood, NodeRow, PathResult, Provenance, Truncation,
};

pub use tags::{ScopedTag, TagAffiliation, TagAffiliations, TagRow};
pub const MAX_QUERY_BYTES: usize = 1_048_576;
pub const MAX_PAGE_SIZE: u16 = 2_000;
pub const MAX_EDGES: usize = 2_000;
pub const MAX_NODES: usize = 500;
pub const MAX_DEPTH: u8 = 3;

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("graph command failed: {0}")]
    Transport(String),
    #[error("graph server rejected the command: {0}")]
    Command(String),
    #[error("projection delivery ownership was superseded")]
    OwnershipLost,
    #[error("projection fence has not been claimed")]
    ProjectionNotClaimed,
    #[error("graph command timed out")]
    Timeout,
    #[error("graph name is invalid")]
    GraphName,
    #[error("graph request admission closed")]
    AdmissionClosed,
    #[error("graph traversal or page limit is invalid")]
    Limits,
    #[error("graph query exceeds its byte bound")]
    QueryTooLarge,
    #[error("graph allocation failed")]
    Allocation,
    #[error("graph row is invalid")]
    InvalidRow,
    #[error("replacement edge is not incident to its owner")]
    UnrelatedEdge,
    #[error("replacement owner does not exist")]
    MissingOwner,
    #[error("graph response is malformed or exceeds its row bound")]
    Response,
    #[error("path search exceeded its traversal budget")]
    Truncated,
}

pub trait GraphBackend: Send + Sync {
    fn target_identity(&self) -> Option<(&str, &str)>;
    fn claim_projection(
        &self,
        generation: i64,
    ) -> impl Future<Output = Result<(), GraphError>> + Send;
    fn clear_tags(&self, key: &NodeKey) -> impl Future<Output = Result<(), GraphError>> + Send;
    fn upsert_tag(
        &self,
        key: &NodeKey,
        tag: &TagRow,
    ) -> impl Future<Output = Result<(), GraphError>> + Send;
    fn tag_affiliations(
        &self,
        key: &NodeKey,
        limit: u16,
    ) -> impl Future<Output = Result<TagAffiliations, GraphError>> + Send;
    fn upsert_node(&self, node: &NodeRow) -> impl Future<Output = Result<(), GraphError>> + Send;
    fn delete_node(&self, key: &NodeKey) -> impl Future<Output = Result<(), GraphError>> + Send;
    /// Replaces all incident edges, subdividing bounded queries when necessary.
    ///
    /// Readers may observe no edges after the clear and only an applied prefix
    /// between appends. Failure leaves that partial set until a complete replay.
    /// A caller must acknowledge durable work only after success.
    fn replace_edges(
        &self,
        key: &NodeKey,
        edges: &[EdgeRow],
    ) -> impl Future<Output = Result<(), GraphError>> + Send;
    /// Replaces the complete incident set from ordered chunks of at most 2,000 edges.
    ///
    /// Removes stale incident edges once, then applies each chunk in order. This is
    /// not atomic: readers can see no edges or only the applied prefix between
    /// queries, as with the previous projector's clear-then-stream delivery.
    /// Each emitted query is bounded to 1 MiB; adapters may subdivide a chunk.
    /// Errors or cancellation leave a partial set; replay must restart the complete
    /// replacement. A caller must acknowledge durable work only after success.
    /// Producers must emit only edges incident to the owner. A bad later page is
    /// rejected after earlier pages have already become visible to readers.
    fn replace_edges_paged<Pages, E>(
        &self,
        key: &NodeKey,
        pages: Pages,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        Pages: futures_util::Stream<Item = Result<Vec<EdgeRow>, E>> + Send,
        E: From<GraphError> + Send;
    fn clear(&self) -> impl Future<Output = Result<(), GraphError>> + Send;
    fn neighbors(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
    ) -> impl Future<Output = Result<GraphNeighborhood, GraphError>> + Send;
    fn path(
        &self,
        from: &NodeKey,
        to: &NodeKey,
        limits: GraphLimits,
    ) -> impl Future<Output = Result<PathResult, GraphError>> + Send;
    fn blast_radius(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
    ) -> impl Future<Output = Result<GraphNeighborhood, GraphError>> + Send;
    fn node_keys(
        &self,
        after: Option<&NodeKey>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<NodeKey>, GraphError>> + Send;
    fn edge_keys(
        &self,
        after: Option<&EdgeKey>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<EdgeKey>, GraphError>> + Send;
}
