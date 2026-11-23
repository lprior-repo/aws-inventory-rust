use std::path::PathBuf;

pub use inventory_core::inventory::ScopeOutcome;
use thiserror::Error;

#[cfg(test)]
use catalog_tool as fixture_catalog;
mod call;
mod config;
#[cfg(test)]
#[path = "../../catalog-tool/src/overlay/builtin/family_a_c/consumer_fixture.rs"]
pub(crate) mod consumer_fixture;
mod control;
mod credentials;
mod execution;
mod input;
#[cfg(test)]
mod pinned_model;
mod record;
mod request;
mod resource_projection;
mod row_policy;
#[cfg(test)]
mod test_cbor;

pub use control::{ProgressEvent, SweepControl, SweepMetrics};
pub use credentials::{CredentialFailure, CredentialSource};
pub use execution::run;

#[derive(Clone, Debug)]
pub struct SweepConfig {
    pub region: Option<String>,
    pub max_pages: u32,
    pub concurrency: usize,
    pub resume: bool,
    pub endpoint: Option<String>,
    pub ca_bundle: Option<PathBuf>,
    /// Optional slice timeout in seconds. When set, the sweep runs for at most
    /// this duration per invocation, then stops and leaves remaining work in the
    /// queue for the next slice. A later invocation with resume=true continues
    /// where this slice stopped.
    pub slice_timeout_secs: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeOutcomes {
    pub complete: u64,
    pub partial: u64,
    pub inaccessible: u64,
    pub unsupported: u64,
    pub failed: u64,
    pub unobserved: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SweepSummary {
    pub resource_writes: u64,
    pub rows: u64,
    pub scopes: u64,
    pub outcomes: ScopeOutcomes,
    pub graph_pending: u64,
    /// Scopes still queued when a time-bounded slice ended; zero for a drained sweep.
    pub work_remaining: u64,
    pub metrics: SweepMetrics,
}

impl ScopeOutcomes {
    pub(crate) fn record(&mut self, outcome: ScopeOutcome) {
        let count = match outcome {
            ScopeOutcome::Complete => &mut self.complete,
            ScopeOutcome::Partial => &mut self.partial,
            ScopeOutcome::Inaccessible => &mut self.inaccessible,
            ScopeOutcome::Unsupported => &mut self.unsupported,
            ScopeOutcome::Failed => &mut self.failed,
            ScopeOutcome::Unobserved => &mut self.unobserved,
        };
        *count = count.saturating_add(1);
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SweepError {
    #[error("invalid sweep configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("inventory domain value is invalid: {0}")]
    Inventory(#[from] inventory_core::inventory::InventoryError),
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),
    #[error("record construction failed: {0}")]
    Build(#[from] inventory_core::record::BuildError),
    #[error("allocation failed: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
    #[error("runtime catalog error: {0}")]
    Catalog(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error("catalog model access failed: {0}")]
    Model(#[from] inventory_core::catalog::CatalogError),
    #[error("API executor error: {0}")]
    Api(#[from] inventory_engine::limiter::ApiError),
    #[error("inventory store error: {0}")]
    Store(#[from] inventory_engine::inventory::StoreError),
    #[error("AWS profile credential error: {0}")]
    Credentials(String),
    #[error("AWS wire request error: {0}")]
    Encode(#[from] aws_wire::request::EncodeError),
    #[error("AWS wire response error: {0}")]
    Decode(#[from] aws_wire::response::DecodeError),
    #[error("AWS transport error: {0}")]
    Transport(#[from] aws_wire::transport::TransportError),
    #[error("AWS signing error: {0}")]
    Signing(#[from] aws_wire::sign::Error),
    #[error("sweep operation failed: {0}")]
    Operation(&'static str),
}
