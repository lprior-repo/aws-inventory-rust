#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProbeError {
    #[error("invalid probe arguments: {0}")]
    Invalid(&'static str),
    #[error("comparison pair was supplied more than once: {0}")]
    DuplicatePair(String),
    #[error("probe file access failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("probe receipt {path} could not be published: {source}")]
    Receipt {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("probe allocation failed: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
    #[error("probe catalog is invalid: {0}")]
    Catalog(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error("probe region is invalid: {0}")]
    Region(#[from] inventory_core::inventory::InventoryError),
    #[error("probe credentials are invalid: {0}")]
    Credentials(#[from] aws_wire::sign::Error),
    #[error("probe transport setup failed: {0}")]
    Transport(#[from] aws_wire::transport::TransportError),
    #[error("probe executor setup failed: {0}")]
    Executor(#[from] inventory_engine::limiter::ApiError),
    #[error("probe setup task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("probe report exceeds its bounded size")]
    Report(#[from] std::fmt::Error),
    #[error("probe total overflow")]
    TotalOverflow,
    #[error("probe child scheduling failed: {0}")]
    ChildSchedule(#[source] inventory_sweep::SweepError),
    #[error("comparison call failed: {0}")]
    Sweep(#[from] inventory_sweep::SweepError),
    #[error("comparison record is invalid: {0}")]
    Record(#[from] inventory_core::record::RecordError),
    #[error("comparison model is invalid: {0}")]
    Model(#[from] inventory_core::catalog::CatalogError),
    #[error("comparison overlay is invalid: {0}")]
    OverlayName(#[from] catalog_tool::overlay::NameError),
    #[error("comparison overlay cannot be rendered: {0}")]
    Overlay(#[from] catalog_tool::overlay::OverlayError),
}
