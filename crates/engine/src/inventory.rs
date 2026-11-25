mod authority;
mod commit;
mod completion;
mod connection;
pub use connection::StoreState;
mod facets;
mod legacy;
pub mod postgres;
pub use postgres::{DatabaseUrl, PgStore, PgWriteConfig};
mod projection;
mod projection_delivery;
mod projection_identity;
mod projection_outbox;
mod projection_read;
pub use projection_delivery::BinaryDelivery;
pub use projection_identity::ProjectionIdentity;
mod reference_candidates;
mod references;
mod registry;
mod relation_permissions;
mod schema;
mod store_api;
mod sweep_history;
mod tag_mapping;
mod typed;
mod unobserved;
mod validation;
mod work;
pub use work::persisted_scope as decode_scope_key;
#[cfg(test)]
mod scope_key_tests;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_core::inventory::{
    AccountId, Edge, KindSchema, Observation, ScopeOutcome, UnresolvedReference,
};
pub use inventory_core::inventory::{Tag, TagCoverage, TagObservation};
pub use projection::{ProjectionError, ProjectionReport, run_projection};
pub use relation_permissions::admits_reference;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

pub use projection_read::{
    ProjectionEdgeCursor, ProjectionEdgeHeader, ProjectionEdgePage, ProjectionResource,
    ProjectionSnapshot, ProjectionState, ProjectionTag, ProjectionTagCursor, ProjectionTagPage,
};
pub use store_api::{InventoryStore, ProjectionDelivery, ProjectionSnapshotRead};
pub use sweep_history::{LastSweep, RunStart, decide_start, last_completed_sweep, plan_run};
pub use unobserved::UnobservedScope;
pub use work::{
    ChangeStats, CommitResult, PendingProjection, ProjectionAck, SweepId, SweepSummary, WorkLease,
    WorkSpec,
};

const MAX_CATALOG_BYTES: usize = inventory_core::catalog::MAX_CATALOG_BYTES;
const MAX_TOKEN_BYTES: usize = 65_536;
const MAX_BINDING_BYTES: usize = 1_048_576;
const MAX_PROJECTION_ROWS: i64 = 100_000;
pub const MAX_OUTCOME_DETAIL_BYTES: usize = 1_024;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("SQLite storage operation failed: {0}")]
    Sqlite(#[from] sqlx::Error),
    #[error("another projector owns this inventory database")]
    DeliveryBusy,
    #[error("projection ownership file operation failed: {0}")]
    OwnershipIo(#[from] std::io::Error),
    #[error("schema or stored data is invalid: {0}")]
    Invalid(&'static str),
    #[error("compiled catalog is invalid: {0}")]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error("runtime catalog capability metadata is invalid: {0}")]
    RuntimeCatalog(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error("database uses an unsupported schema version: {found}")]
    SchemaVersion { found: i64 },
    #[error("inventory schema initialization did not complete; create it again to resume")]
    InitializationIncomplete,
    #[error("catalog digest differs from the immutable database catalog")]
    CatalogMismatch,
    #[error("stale or invalid work lease")]
    StaleLease,
    #[error("record encoding failed: {0}")]
    Record(#[from] inventory_core::record::RecordError),
    #[error("inventory domain value is invalid: {0}")]
    Domain(#[from] inventory_core::inventory::InventoryError),
    #[error("another sweep is active; resume its persisted identity before seeding new work")]
    ActiveSweepConflict,
    #[error("work lease generation is exhausted")]
    LeaseGenerationOverflow,
    #[error("multiple active sweep identities exist in the work queue")]
    MultipleActiveSweeps,
    #[error("sweep has live leases; cannot resume until leases expire")]
    LiveLease,
    #[error("sweep manifest account mismatch")]
    AccountMismatch(
        inventory_core::inventory::AccountId,
        Option<inventory_core::inventory::AccountId>,
    ),
}

#[derive(Clone, Debug)]
pub struct SchemaRegistry {
    schemas: Arc<[Arc<KindSchema>]>,
    digest: [u8; 32],
    operations: Arc<BTreeSet<(String, String)>>,
    permissions: relation_permissions::Permissions,
}

pub struct BinaryStore {
    writer: SqlitePool,
    reader: SqlitePool,
    writer_turn: Arc<tokio::sync::Mutex<()>>,
    reader_shares_writer: bool,
    pub(crate) registry: SchemaRegistry,
}

pub struct PageCommit {
    pub lease: WorkLease,
    pub observations: Vec<Observation>,
    pub edges: Vec<Edge>,
    pub unresolved: Vec<UnresolvedReference>,
    pub tags: Vec<TagObservation>,
    pub next_token: Option<Vec<u8>>,
    pub continuation_digest: Option<[u8; 32]>,
    pub children: Vec<WorkSpec>,
    pub done: bool,
    pub observed_at_ms: i64,
}

impl BinaryStore {
    pub async fn create(path: &Path, registry: SchemaRegistry) -> Result<Self, StoreError> {
        connection::create(path, registry).await
    }

    pub async fn open(path: &Path) -> Result<Self, StoreError> {
        connection::open(path).await
    }

    /// Classifies `path` so callers choose `open` for initialized databases and
    /// `create` for absent or interrupted-initialization files.
    pub async fn probe(path: &Path) -> Result<StoreState, StoreError> {
        connection::probe(path).await
    }

    pub async fn open_with_catalog(path: &Path, catalog: &[u8]) -> Result<Self, StoreError> {
        relation_permissions::open_with_catalog(path, catalog).await
    }

    pub async fn request_graph_rebuild(&self) -> Result<(), StoreError> {
        commit::request_graph_rebuild(self).await
    }

    pub async fn unattributed_authority_count(&self) -> Result<u64, StoreError> {
        let _turn = self.writer_turn.lock().await;
        legacy::count(&self.writer).await
    }

    pub async fn seed_work(&self, work: WorkSpec) -> Result<(), StoreError> {
        work::seed(self, work).await
    }
    pub async fn seed_works(&self, works: &[WorkSpec]) -> Result<(), StoreError> {
        work::seed_works(self, works).await
    }

    pub async fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        work::claim(self, sweep, now_ms, lease_ms).await
    }

    pub async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        work::commit_page(self, page).await
    }

    pub async fn commit_page_keeping(
        &self,
        page: PageCommit,
        lease_ms: i64,
    ) -> Result<(CommitResult, Option<WorkLease>), StoreError> {
        work::commit_page_keeping(self, page, lease_ms).await
    }

    pub async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
        detail: Option<&str>,
    ) -> Result<ChangeStats, StoreError> {
        work::finish_scope(self, lease, outcome, observed_at_ms, detail).await
    }

    pub async fn drain_projection(&self, limit: u16) -> Result<Vec<PendingProjection>, StoreError> {
        projection_outbox::drain_projection(self, limit).await
    }

    pub async fn ack_projection(&self, ack: ProjectionAck) -> Result<u64, StoreError> {
        projection_outbox::ack_projection(self, ack).await
    }
    pub async fn projection_state(&self) -> Result<ProjectionState, StoreError> {
        projection_read::state(self).await
    }

    pub async fn projection_resources(
        &self,
        after: Option<inventory_core::inventory::ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        projection_read::resources(self, after, limit).await
    }

    pub async fn projection_edges(
        &self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        projection_read::edges(self, after, limit).await
    }
    pub async fn projection_resource(
        &self,
        key: &inventory_core::inventory::ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        projection_read::resource(self, key).await
    }

    pub async fn projection_edges_for(
        &self,
        key: &inventory_core::inventory::ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        projection_read::edges_for(self, key, after, limit).await
    }

    pub async fn begin_projection_snapshot(&self) -> Result<ProjectionSnapshot, StoreError> {
        projection_read::begin_snapshot(self).await
    }
    pub async fn validate_catalog(&self, catalog: &[u8]) -> Result<(), StoreError> {
        let requested: [u8; 32] = Sha256::digest(catalog).into();
        let _turn = self.read_turn().await;
        let stored: Vec<u8> = sqlx::query("SELECT value FROM metadata WHERE key='catalog_digest'")
            .fetch_one(&self.reader)
            .await?
            .try_get(0)?;
        if stored.as_slice() != requested {
            return Err(StoreError::CatalogMismatch);
        }
        Ok(())
    }

    pub async fn resume_sweep(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> Result<(), StoreError> {
        work::resume_sweep(self, sweep, account).await
    }
    pub async fn discover_active_sweep(&self) -> Result<Option<SweepId>, StoreError> {
        work::discover_active_sweep(self).await
    }
    pub async fn sweep_manifest_account(
        &self,
        sweep: &SweepId,
    ) -> Result<Option<String>, StoreError> {
        work::sweep_manifest_account(self, sweep).await
    }
    pub async fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &str,
    ) -> Result<(), StoreError> {
        work::create_sweep_manifest(self, sweep, account).await
    }
    pub async fn unobserved_scopes(
        &self,
        sweep: &SweepId,
        now_ms: i64,
    ) -> Result<Vec<UnobservedScope>, StoreError> {
        unobserved::list(self, sweep, now_ms).await
    }
    pub async fn sweep_summary(&self, sweep: &SweepId) -> Result<SweepSummary, StoreError> {
        projection_outbox::sweep_summary(self, sweep).await
    }
}

fn schema_sets_match(given: &[Arc<KindSchema>], checked: &[Arc<KindSchema>]) -> bool {
    given.len() == checked.len()
        && given
            .iter()
            .all(|left| checked.iter().any(|right| schemas_equal(left, right)))
}

fn schemas_equal(left: &KindSchema, right: &KindSchema) -> bool {
    left.kind() == right.kind()
        && left.table() == right.table()
        && left.columns().len() == right.columns().len()
        && left
            .columns()
            .iter()
            .zip(right.columns())
            .all(|(a, b)| a.name() == b.name() && a.path() == b.path() && a.scalar() == b.scalar())
}
