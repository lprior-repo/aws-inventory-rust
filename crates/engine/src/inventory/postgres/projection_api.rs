use super::{
    PgDelivery, PgProjectionSnapshot, PgStore, outbox, projection_delivery, projection_read,
    projection_state,
};
use crate::inventory::{
    ChangeStats, CommitResult, InventoryStore, PageCommit, PendingProjection, ProjectionAck,
    ProjectionEdgeCursor, ProjectionEdgePage, ProjectionIdentity, ProjectionResource,
    ProjectionSnapshotRead, ProjectionState, SchemaRegistry, StoreError, SweepId, SweepSummary,
    WorkLease, WorkSpec,
};
use inventory_core::inventory::{AccountId, ResourceKey, ScopeOutcome};

impl InventoryStore for PgStore {
    type Snapshot = PgProjectionSnapshot;
    type Delivery = PgDelivery;

    async fn acquire_projection_delivery(&self) -> Result<Self::Delivery, StoreError> {
        projection_delivery::acquire(self).await
    }
    async fn read_projection_identity(&self) -> Result<Option<ProjectionIdentity>, StoreError> {
        projection_state::identity(self).await
    }
    async fn write_projection_identity(
        &self,
        identity: &ProjectionIdentity,
    ) -> Result<(), StoreError> {
        projection_state::write_identity(self, identity).await
    }
    async fn mark_projection_rebuild_required(&self) -> Result<(), StoreError> {
        projection_state::mark_rebuild(self).await
    }
    async fn request_graph_rebuild(&self) -> Result<(), StoreError> {
        outbox::request_rebuild(self).await
    }
    async fn validate_catalog(&self, catalog: &[u8]) -> Result<(), StoreError> {
        PgStore::validate_catalog(self, catalog).await
    }
    async fn seed_works(&self, works: &[WorkSpec]) -> Result<(), StoreError> {
        PgStore::seed_works(self, works).await
    }
    async fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        PgStore::claim(self, sweep, now_ms, lease_ms).await
    }
    async fn claim_batch(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
        limit: u16,
    ) -> Result<Vec<WorkLease>, StoreError> {
        PgStore::claim_batch(self, sweep, now_ms, lease_ms, limit).await
    }
    async fn claim_with_depth(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        PgStore::claim_with_depth(self, sweep, now_ms, lease_ms).await
    }
    async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        PgStore::commit_page(self, page).await
    }
    async fn commit_page_keeping(
        &self,
        page: PageCommit,
        lease_ms: i64,
    ) -> Result<(CommitResult, Option<WorkLease>), StoreError> {
        PgStore::commit_page_keeping(self, page, lease_ms).await
    }
    async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
        detail: Option<&str>,
    ) -> Result<ChangeStats, StoreError> {
        PgStore::finish_scope(self, lease, outcome, observed_at_ms, detail).await
    }
    async fn release_lease(&self, lease: WorkLease) -> Result<(), StoreError> {
        PgStore::release_lease(self, lease).await
    }
    async fn discover_active_sweep(&self) -> Result<Option<SweepId>, StoreError> {
        PgStore::discover_active_sweep(self).await
    }
    async fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> Result<(), StoreError> {
        PgStore::create_sweep_manifest(self, sweep, account.as_str()).await
    }
    async fn resume_sweep(&self, sweep: &SweepId, account: &AccountId) -> Result<(), StoreError> {
        PgStore::resume_sweep(self, sweep, account).await
    }
    async fn sweep_summary(&self, sweep: &SweepId) -> Result<SweepSummary, StoreError> {
        PgStore::sweep_summary(self, sweep).await
    }
    async fn drain_projection(&self, limit: u16) -> Result<Vec<PendingProjection>, StoreError> {
        outbox::drain(self, limit).await
    }
    async fn ack_projection(&self, ack: ProjectionAck) -> Result<u64, StoreError> {
        outbox::ack(self, ack).await
    }
    async fn begin_projection_snapshot(&self) -> Result<Self::Snapshot, StoreError> {
        projection_read::begin(self).await
    }
    async fn projection_state(&self) -> Result<ProjectionState, StoreError> {
        projection_state::state(self).await
    }
    async fn projection_resources(
        &self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        let mut snapshot = projection_read::begin(self).await?;
        let resources = snapshot.resources(after, limit).await?;
        snapshot.finish().await?;
        Ok(resources)
    }
    async fn projection_resource(
        &self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        let mut snapshot = projection_read::begin(self).await?;
        let resource = snapshot.resource(key).await?;
        snapshot.finish().await?;
        Ok(resource)
    }
    async fn projection_edges(
        &self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        let mut snapshot = projection_read::begin(self).await?;
        let edges = snapshot.edges(after, limit).await?;
        snapshot.finish().await?;
        Ok(edges)
    }
    async fn projection_edges_for(
        &self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        let mut snapshot = projection_read::begin(self).await?;
        let edges = snapshot.edges_for(key, after, limit).await?;
        snapshot.finish().await?;
        Ok(edges)
    }
    fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }
}
