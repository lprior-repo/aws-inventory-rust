use super::*;
use crate::inventory::{BinaryStore, ProjectionSnapshot, projection_delivery, projection_identity};
impl InventoryStore for BinaryStore {
    type Snapshot = ProjectionSnapshot;
    type Delivery = crate::inventory::BinaryDelivery;

    async fn acquire_projection_delivery(&self) -> Result<Self::Delivery, StoreError> {
        projection_delivery::acquire(self).await
    }
    async fn read_projection_identity(&self) -> Result<Option<ProjectionIdentity>, StoreError> {
        projection_identity::read(self).await
    }
    async fn write_projection_identity(
        &self,
        identity: &ProjectionIdentity,
    ) -> Result<(), StoreError> {
        projection_identity::write(self, identity).await
    }
    async fn mark_projection_rebuild_required(&self) -> Result<(), StoreError> {
        projection_identity::mark_rebuild(self).await
    }

    async fn validate_catalog(&self, catalog: &[u8]) -> Result<(), StoreError> {
        BinaryStore::validate_catalog(self, catalog).await
    }
    async fn seed_works(&self, works: &[WorkSpec]) -> Result<(), StoreError> {
        BinaryStore::seed_works(self, works).await
    }
    async fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        BinaryStore::claim(self, sweep, now_ms, lease_ms).await
    }
    async fn claim_with_depth(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        BinaryStore::claim_with_depth(self, sweep, now_ms, lease_ms).await
    }
    async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        BinaryStore::commit_page(self, page).await
    }
    async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
    ) -> Result<ChangeStats, StoreError> {
        BinaryStore::finish_scope(self, lease, outcome, observed_at_ms).await
    }
    async fn release_lease(&self, lease: WorkLease) -> Result<(), StoreError> {
        BinaryStore::release_lease(self, lease).await
    }
    async fn discover_active_sweep(&self) -> Result<Option<SweepId>, StoreError> {
        BinaryStore::discover_active_sweep(self).await
    }
    async fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> Result<(), StoreError> {
        BinaryStore::create_sweep_manifest(self, sweep, account.as_str()).await
    }
    async fn resume_sweep(&self, sweep: &SweepId, account: &AccountId) -> Result<(), StoreError> {
        BinaryStore::resume_sweep(self, sweep, account).await
    }
    async fn sweep_summary(&self, sweep: &SweepId) -> Result<SweepSummary, StoreError> {
        BinaryStore::sweep_summary(self, sweep).await
    }
    async fn drain_projection(&self, limit: u16) -> Result<Vec<PendingProjection>, StoreError> {
        BinaryStore::drain_projection(self, limit).await
    }
    async fn ack_projection(&self, ack: ProjectionAck) -> Result<u64, StoreError> {
        BinaryStore::ack_projection(self, ack).await
    }
    async fn begin_projection_snapshot(&self) -> Result<Self::Snapshot, StoreError> {
        BinaryStore::begin_projection_snapshot(self).await
    }
    async fn projection_state(&self) -> Result<ProjectionState, StoreError> {
        BinaryStore::projection_state(self).await
    }
    async fn projection_resources(
        &self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        BinaryStore::projection_resources(self, after, limit).await
    }
    async fn projection_resource(
        &self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        BinaryStore::projection_resource(self, key).await
    }
    async fn projection_edges(
        &self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        BinaryStore::projection_edges(self, after, limit).await
    }
    async fn projection_edges_for(
        &self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        BinaryStore::projection_edges_for(self, key, after, limit).await
    }
    fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }
}

impl ProjectionSnapshotRead for ProjectionSnapshot {
    async fn resources(
        &mut self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        ProjectionSnapshot::resources(self, after, limit).await
    }
    async fn resource(
        &mut self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        ProjectionSnapshot::resource(self, key).await
    }
    async fn edges(
        &mut self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        ProjectionSnapshot::edges(self, after, limit).await
    }
    async fn edges_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        ProjectionSnapshot::edges_for(self, key, after, limit).await
    }
    async fn tags_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionTagCursor>,
        limit: u16,
    ) -> Result<ProjectionTagPage, StoreError> {
        ProjectionSnapshot::tags_for(self, key, after, limit).await
    }
    async fn finish(self) -> Result<(), StoreError> {
        ProjectionSnapshot::finish(self).await
    }
}
