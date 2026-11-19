mod authority;
mod commit;
mod completion;
mod connection;
mod facets;
mod legacy;
mod projection;
mod projection_outbox;
mod projection_read;
mod reference_candidates;
mod references;
mod relation_permissions;
mod schema;
mod typed;
mod validation;
mod work;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_core::inventory::{
    AccountId, Edge, KindSchema, Observation, ScopeKey, ScopeOutcome, UnresolvedReference,
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
pub use work::{
    ChangeStats, CommitResult, PendingProjection, ProjectionAck, SweepId, SweepSummary, WorkLease,
    WorkSpec,
};

const MAX_CATALOG_BYTES: usize = inventory_core::catalog::MAX_CATALOG_BYTES;
const MAX_TOKEN_BYTES: usize = 65_536;
const MAX_BINDING_BYTES: usize = 1_048_576;
const MAX_PROJECTION_ROWS: i64 = 100_000;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("SQLite storage operation failed: {0}")]
    Sqlite(#[from] sqlx::Error),
    #[error("schema or stored data is invalid: {0}")]
    Invalid(&'static str),
    #[error("compiled catalog is invalid: {0}")]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error("runtime catalog capability metadata is invalid: {0}")]
    RuntimeCatalog(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error(
        "database uses schema version {found}; expected schema16 (schema14/15 upgrade on open)"
    )]
    SchemaVersion { found: i64 },
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

impl SchemaRegistry {
    pub fn new(catalog: &[u8], schemas: Vec<Arc<KindSchema>>) -> Result<Self, StoreError> {
        if catalog.is_empty() || catalog.len() > MAX_CATALOG_BYTES || schemas.is_empty() {
            return Err(StoreError::Invalid("catalog size or kind schema count"));
        }
        let runtime = RuntimeCatalog::from_bytes(catalog)?;
        let operations = runtime
            .plans()
            .iter()
            .map(|plan| {
                (
                    plan.service_name().to_owned(),
                    plan.operation_name().to_owned(),
                )
            })
            .collect();
        let permissions = relation_permissions::Permissions::new(catalog, &runtime)?;
        let registry = Self::stored(
            Sha256::digest(catalog).into(),
            schemas,
            operations,
            permissions,
        )?;
        if !schema_sets_match(&registry.schemas, runtime.schemas()) {
            return Err(StoreError::Invalid(
                "kind schemas differ from checked runtime catalog",
            ));
        }
        Ok(registry)
    }

    fn stored(
        digest: [u8; 32],
        schemas: Vec<Arc<KindSchema>>,
        operations: BTreeSet<(String, String)>,
        permissions: relation_permissions::Permissions,
    ) -> Result<Self, StoreError> {
        let mut ordered = schemas;
        ordered.sort_unstable_by(|left, right| left.kind().cmp(right.kind()));
        if ordered.is_empty()
            || ordered
                .windows(2)
                .any(|pair| matches!(pair, [left, right] if left.kind() == right.kind()))
        {
            return Err(StoreError::Invalid("missing or duplicate kind schema"));
        }
        Ok(Self {
            schemas: ordered.into(),
            digest,
            operations: Arc::new(operations),
            permissions,
        })
    }

    pub fn schemas(&self) -> &[Arc<KindSchema>] {
        &self.schemas
    }
    fn reference_target(
        &self,
        account: inventory_core::inventory::AccountId,
        region: inventory_core::inventory::Region,
        kind: inventory_core::inventory::Kind,
        path: Vec<inventory_core::record::FieldId>,
        value: inventory_core::inventory::ResourceId,
    ) -> Result<inventory_core::inventory::ReferenceTarget, StoreError> {
        if path.is_empty() {
            let schema = self
                .schemas
                .iter()
                .find(|schema| schema.kind() == &kind)
                .ok_or(StoreError::Invalid(
                    "reference target kind is absent from schema registry",
                ))?;
            Ok(inventory_core::inventory::ReferenceTarget::root_text(
                account, region, schema, value,
            )?)
        } else {
            Ok(inventory_core::inventory::ReferenceTarget::new(
                account, region, kind, path, value,
            )?)
        }
    }
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(crate) fn operations(&self) -> &BTreeSet<(String, String)> {
        &self.operations
    }
    pub(crate) fn allows(&self, scope: &ScopeKey) -> bool {
        self.operations.contains(&(
            scope.service().as_str().to_owned(),
            scope.operation().as_str().to_owned(),
        ))
    }
}
pub struct BinaryStore {
    pub(crate) writer: SqlitePool,
    pub(crate) reader: SqlitePool,
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

    pub async fn open_with_catalog(path: &Path, catalog: &[u8]) -> Result<Self, StoreError> {
        relation_permissions::open_with_catalog(path, catalog).await
    }

    pub async fn unattributed_authority_count(&self) -> Result<u64, StoreError> {
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

    pub async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
    ) -> Result<ChangeStats, StoreError> {
        work::finish_scope(self, lease, outcome, observed_at_ms).await
    }

    pub async fn drain_projection(&self, limit: u16) -> Result<Vec<PendingProjection>, StoreError> {
        projection_outbox::drain_projection(self, limit).await
    }

    /// Returns the number of durable outbox rows actually acknowledged (0 or 1).
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
