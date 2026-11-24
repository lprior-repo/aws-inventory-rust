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
    #[error("PostgreSQL connection failed")]
    PostgresConnect,
    #[error("PostgreSQL authentication failed")]
    PostgresAuthentication,
    #[error("PostgreSQL database does not exist")]
    PostgresDatabase,
    #[error("PostgreSQL inventory schema is missing")]
    PostgresSchema,
    #[error("unsupported PostgreSQL inventory schema; expected 14")]
    PostgresVersion,
    #[error("invalid stored resource row")]
    StoredRow,
    #[error("page limit must be in 1..=1000")]
    PageLimit,
    #[error(transparent)]
    Backend(#[from] inventory_graph::GraphError),
    #[error("resource row exceeds the 10 MiB page budget")]
    PageBudget,
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
