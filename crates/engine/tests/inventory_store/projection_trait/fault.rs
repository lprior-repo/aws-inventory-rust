use inventory_core::inventory::{AccountId, ResourceId, ResourceKey, ScopeOutcome};
use inventory_engine::inventory::{
    BinaryStore, ChangeStats, CommitResult, InventoryStore, PageCommit, PendingProjection,
    ProjectionAck, ProjectionEdgeCursor, ProjectionEdgePage, ProjectionIdentity,
    ProjectionResource, ProjectionSnapshotRead, ProjectionState, ProjectionTagCursor,
    ProjectionTagPage, SchemaRegistry, StoreError, SweepId, SweepSummary, WorkLease, WorkSpec,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy)]
pub(super) enum Fault {
    Tags,
    Resources,
    ForeignEdges,
}

pub(super) struct FaultStore {
    pub(super) inner: BinaryStore,
    fault: Fault,
    pub(super) reads: Arc<AtomicUsize>,
    pub(super) acknowledgements: Arc<AtomicUsize>,
}

impl FaultStore {
    pub(super) fn new(inner: BinaryStore, fault: Fault) -> Self {
        Self {
            inner,
            fault,
            reads: Arc::new(AtomicUsize::new(0)),
            acknowledgements: Arc::new(AtomicUsize::new(0)),
        }
    }
}

pub(super) struct FaultSnapshot<S> {
    inner: S,
    fault: Fault,
    reads: Arc<AtomicUsize>,
}

fn count(counter: &AtomicUsize) -> Result<(), StoreError> {
    counter
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
            value.checked_add(1)
        })
        .map(|_| ())
        .map_err(|_| StoreError::Invalid("test page counter overflow"))
}

impl InventoryStore for FaultStore {
    type Snapshot = FaultSnapshot<<BinaryStore as InventoryStore>::Snapshot>;
    type Delivery = <BinaryStore as InventoryStore>::Delivery;

    async fn acquire_projection_delivery(&self) -> Result<Self::Delivery, StoreError> {
        self.inner.acquire_projection_delivery().await
    }
    async fn read_projection_identity(&self) -> Result<Option<ProjectionIdentity>, StoreError> {
        self.inner.read_projection_identity().await
    }
    async fn write_projection_identity(
        &self,
        identity: &ProjectionIdentity,
    ) -> Result<(), StoreError> {
        self.inner.write_projection_identity(identity).await
    }
    async fn mark_projection_rebuild_required(&self) -> Result<(), StoreError> {
        self.inner.mark_projection_rebuild_required().await
    }
    async fn request_graph_rebuild(&self) -> Result<(), StoreError> {
        self.inner.request_graph_rebuild().await
    }
    async fn validate_catalog(&self, catalog: &[u8]) -> Result<(), StoreError> {
        self.inner.validate_catalog(catalog).await
    }
    async fn seed_works(&self, works: &[WorkSpec]) -> Result<(), StoreError> {
        self.inner.seed_works(works).await
    }
    async fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        self.inner.claim(sweep, now_ms, lease_ms).await
    }
    async fn claim_with_depth(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        self.inner.claim_with_depth(sweep, now_ms, lease_ms).await
    }
    async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        self.inner.commit_page(page).await
    }
    async fn commit_page_keeping(
        &self,
        page: PageCommit,
        lease_ms: i64,
    ) -> Result<(CommitResult, Option<WorkLease>), StoreError> {
        self.inner.commit_page_keeping(page, lease_ms).await
    }
    async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
        detail: Option<&str>,
    ) -> Result<ChangeStats, StoreError> {
        self.inner
            .finish_scope(lease, outcome, observed_at_ms, detail)
            .await
    }
    async fn release_lease(&self, lease: WorkLease) -> Result<(), StoreError> {
        self.inner.release_lease(lease).await
    }
    async fn discover_active_sweep(&self) -> Result<Option<SweepId>, StoreError> {
        self.inner.discover_active_sweep().await
    }
    async fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> Result<(), StoreError> {
        InventoryStore::create_sweep_manifest(&self.inner, sweep, account).await
    }
    async fn resume_sweep(&self, sweep: &SweepId, account: &AccountId) -> Result<(), StoreError> {
        self.inner.resume_sweep(sweep, account).await
    }
    async fn sweep_summary(&self, sweep: &SweepId) -> Result<SweepSummary, StoreError> {
        self.inner.sweep_summary(sweep).await
    }
    async fn drain_projection(&self, limit: u16) -> Result<Vec<PendingProjection>, StoreError> {
        self.inner.drain_projection(limit).await
    }
    async fn ack_projection(&self, ack: ProjectionAck) -> Result<u64, StoreError> {
        count(&self.acknowledgements)?;
        self.inner.ack_projection(ack).await
    }
    async fn begin_projection_snapshot(&self) -> Result<Self::Snapshot, StoreError> {
        Ok(FaultSnapshot {
            inner: self.inner.begin_projection_snapshot().await?,
            fault: self.fault,
            reads: Arc::clone(&self.reads),
        })
    }
    async fn projection_state(&self) -> Result<ProjectionState, StoreError> {
        self.inner.projection_state().await
    }
    async fn projection_resources(
        &self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        self.inner.projection_resources(after, limit).await
    }
    async fn projection_resource(
        &self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        self.inner.projection_resource(key).await
    }
    async fn projection_edges(
        &self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        self.inner.projection_edges(after, limit).await
    }
    async fn projection_edges_for(
        &self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        self.inner.projection_edges_for(key, after, limit).await
    }
    fn registry(&self) -> &SchemaRegistry {
        self.inner.registry()
    }
}

impl<S: ProjectionSnapshotRead> ProjectionSnapshotRead for FaultSnapshot<S> {
    async fn resources(
        &mut self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        if matches!(self.fault, Fault::Resources) {
            count(&self.reads)?;
            self.inner.resources(None, limit).await
        } else {
            self.inner.resources(after, limit).await
        }
    }
    async fn resource(
        &mut self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        self.inner.resource(key).await
    }
    async fn edges(
        &mut self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        self.inner.edges(after, limit).await
    }
    async fn edges_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        let mut page = self.inner.edges_for(key, after, limit).await?;
        if matches!(self.fault, Fault::ForeignEdges) {
            count(&self.reads)?;
            if self.reads.load(Ordering::SeqCst) == 3 {
                let mut foreign = key.clone();
                foreign.id = ResourceId::parse("foreign")
                    .map_err(|_| StoreError::Invalid("foreign fixture id"))?;
                let edge = page
                    .edges
                    .first_mut()
                    .ok_or(StoreError::Invalid("missing final edge"))?;
                edge.from = foreign.clone();
                edge.to = foreign;
            }
        }
        Ok(page)
    }
    async fn tags_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionTagCursor>,
        limit: u16,
    ) -> Result<ProjectionTagPage, StoreError> {
        if matches!(self.fault, Fault::Tags) {
            count(&self.reads)?;
            self.inner.tags_for(key, None, limit).await
        } else {
            self.inner.tags_for(key, after, limit).await
        }
    }
    async fn finish(self) -> Result<(), StoreError> {
        self.inner.finish().await
    }
}
