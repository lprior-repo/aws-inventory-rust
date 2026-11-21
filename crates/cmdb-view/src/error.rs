use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Identity(#[from] inventory_core::inventory::InventoryError),
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),
    #[error(
        "unsupported schema version {0}; expected 15, 16, or 17 (open with the writer to migrate schema 14/15/16)"
    )]
    Version(i64),
    #[error("invalid stored resource row")]
    StoredRow,
    #[error("page limit must be in 1..=1000")]
    PageLimit,
    #[error("FalkorDB command failed")]
    Graph(#[source] redis::RedisError),
    #[error("FalkorDB operation timed out")]
    GraphTimeout,
    #[error("invalid FalkorDB graph name")]
    GraphName,
    #[error("FalkorDB response is malformed")]
    GraphResponse,
    #[error("graph traversal limits are out of range")]
    GraphLimits,
    #[error("stored SQL table name is invalid")]
    SqlName,
}
