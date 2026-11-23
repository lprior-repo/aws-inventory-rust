#[path = "store_api/binary.rs"]
mod binary;

use super::{
    ChangeStats, CommitResult, PageCommit, PendingProjection, ProjectionAck, ProjectionEdgeCursor,
    ProjectionEdgePage, ProjectionIdentity, ProjectionResource, ProjectionState,
    ProjectionTagCursor, ProjectionTagPage, SchemaRegistry, StoreError, SweepId, SweepSummary,
    WorkLease, WorkSpec,
};
use inventory_core::inventory::{AccountId, ResourceKey, ScopeOutcome};

pub trait InventoryStore: Send + Sync {
    type Snapshot: ProjectionSnapshotRead;
    type Delivery: ProjectionDelivery;

    fn acquire_projection_delivery(
        &self,
    ) -> impl Future<Output = Result<Self::Delivery, StoreError>> + Send;
    fn read_projection_identity(
        &self,
    ) -> impl Future<Output = Result<Option<ProjectionIdentity>, StoreError>> + Send;
    fn write_projection_identity(
        &self,
        identity: &ProjectionIdentity,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn mark_projection_rebuild_required(
        &self,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;

    fn validate_catalog(
        &self,
        catalog: &[u8],
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn seed_works(&self, works: &[WorkSpec])
    -> impl Future<Output = Result<(), StoreError>> + Send;
    fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> impl Future<Output = Result<Option<WorkLease>, StoreError>> + Send;
    fn claim_with_depth(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> impl Future<Output = Result<Option<WorkLease>, StoreError>> + Send;
    fn commit_page(
        &self,
        page: PageCommit,
    ) -> impl Future<Output = Result<CommitResult, StoreError>> + Send;
    fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
    ) -> impl Future<Output = Result<ChangeStats, StoreError>> + Send;
    fn release_lease(
        &self,
        lease: WorkLease,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn discover_active_sweep(
        &self,
    ) -> impl Future<Output = Result<Option<SweepId>, StoreError>> + Send;
    fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn resume_sweep(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
    fn sweep_summary(
        &self,
        sweep: &SweepId,
    ) -> impl Future<Output = Result<SweepSummary, StoreError>> + Send;
    fn drain_projection(
        &self,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<PendingProjection>, StoreError>> + Send;
    fn ack_projection(
        &self,
        ack: ProjectionAck,
    ) -> impl Future<Output = Result<u64, StoreError>> + Send;
    fn begin_projection_snapshot(
        &self,
    ) -> impl Future<Output = Result<Self::Snapshot, StoreError>> + Send;
    fn projection_state(&self) -> impl Future<Output = Result<ProjectionState, StoreError>> + Send;
    fn projection_resources(
        &self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<ProjectionResource>, StoreError>> + Send;
    fn projection_resource(
        &self,
        key: &ResourceKey,
    ) -> impl Future<Output = Result<Option<ProjectionResource>, StoreError>> + Send;
    fn projection_edges(
        &self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> impl Future<Output = Result<ProjectionEdgePage, StoreError>> + Send;
    fn projection_edges_for(
        &self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> impl Future<Output = Result<ProjectionEdgePage, StoreError>> + Send;
    fn registry(&self) -> &SchemaRegistry;
}

pub trait ProjectionSnapshotRead: Send {
    fn resources(
        &mut self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<ProjectionResource>, StoreError>> + Send;
    fn resource(
        &mut self,
        key: &ResourceKey,
    ) -> impl Future<Output = Result<Option<ProjectionResource>, StoreError>> + Send;
    fn edges(
        &mut self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> impl Future<Output = Result<ProjectionEdgePage, StoreError>> + Send;
    fn edges_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> impl Future<Output = Result<ProjectionEdgePage, StoreError>> + Send;
    fn tags_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionTagCursor>,
        limit: u16,
    ) -> impl Future<Output = Result<ProjectionTagPage, StoreError>> + Send;
    fn finish(self) -> impl Future<Output = Result<(), StoreError>> + Send;
}

pub trait ProjectionDelivery: Send {
    fn generation(&self) -> i64;
    fn release(self) -> impl Future<Output = Result<(), StoreError>> + Send;
}
