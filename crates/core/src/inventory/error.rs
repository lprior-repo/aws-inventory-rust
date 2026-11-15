#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InventoryError {
    #[error("account must contain exactly 12 ASCII digits")]
    Account,
    #[error("invalid regional AWS name")]
    Region,
    #[error("invalid resource kind")]
    Kind,
    #[error("resource identity is empty, contains controls, or exceeds 4096 bytes")]
    Resource,
    #[error("invalid model service directory")]
    Service,
    #[error("invalid modeled operation name")]
    Operation,
    #[error("work branch must contain 1..64 ASCII alphanumeric, underscore or hyphen bytes")]
    Branch,
    #[error("invalid relationship name")]
    Relation,
    #[error("invalid SQL identifier")]
    SqlName,
    #[error("parent belongs to a different account")]
    ParentAccount,
    #[error("negative observation time")]
    Time,
    #[error("display name exceeds 4096 bytes")]
    Name,
    #[error("field path must contain one to eight field IDs")]
    Path,
    #[error("reference requires one to 64 unique candidate kinds including its declared kind")]
    ReferenceKinds,
    #[error("kind contains duplicate column paths")]
    DuplicateColumn,
    #[error("kind exceeds SQLite's 2000-column bound")]
    Columns,
    #[error("table name uses SQLite's reserved prefix")]
    ReservedTable,
    #[error("page size must be between 1 and 1000")]
    PageSize,
    #[error("search text contains NUL or exceeds 256 bytes")]
    Search,
    #[error("tag key must be nonempty, contain no controls, and fit in 128 bytes")]
    TagKey,
    #[error("tag value contains NUL or exceeds 256 bytes")]
    TagValue,
    #[error("resource tag observation exceeds 100 pairs")]
    TagCount,
    #[error("resource tag observation contains a duplicate key")]
    DuplicateTag,
    #[error(transparent)]
    Record(#[from] crate::record::RecordError),
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}
