#[path = "conformance/children.rs"]
mod children;
#[path = "conformance/control.rs"]
mod control;
#[path = "conformance/data.rs"]
mod data;
#[path = "conformance/leases.rs"]
mod leases;
#[path = "conformance/projection.rs"]
mod projection;
#[path = "conformance/references.rs"]
mod references;
#[path = "conformance/resume.rs"]
mod resume;
#[path = "conformance/snapshot.rs"]
mod snapshot;

use super::support::*;
use inventory_core::inventory::{AccountId, ScopeOutcome};
use inventory_engine::inventory::{
    InventoryStore, ProjectionAck, ProjectionSnapshotRead, StoreError, SweepId, SweepSummary,
    WorkLease,
};

pub async fn run<S, F, G>(make: F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    leases::run(&make).await?;
    resume::run(&make).await?;
    control::run(&make).await?;
    children::run(&make).await?;
    data::run(&make).await?;
    references::run(&make).await?;
    projection::run(&make).await?;
    snapshot::run(&make).await
}

async fn seed_claim<S: InventoryStore>(store: &S, sweep: &str, at: i64) -> TestResult<WorkLease> {
    store.seed_works(&[work(sweep, "us-east-1")?]).await?;
    claim(store, sweep, at).await
}

async fn claim<S: InventoryStore>(store: &S, sweep: &str, at: i64) -> TestResult<WorkLease> {
    Ok(store
        .claim(&SweepId::new(sweep.to_owned())?, at, 20)
        .await?
        .ok_or("missing work lease")?)
}

async fn observe<S: InventoryStore>(store: &S, sweep: &str, id: &str, value: &str) -> TestResult {
    let lease = seed_claim(store, sweep, 100).await?;
    let result = store
        .commit_page(page(lease, vec![observation(id, value)?], true, None, 101))
        .await?;
    assert_eq!(result.stats.resources_deleted, 0);
    Ok(())
}

async fn has<S: InventoryStore>(store: &S, id: &str) -> TestResult<bool> {
    Ok(store.projection_resource(&resource(id)?).await?.is_some())
}

async fn summary<S: InventoryStore>(store: &S, sweep: &str) -> TestResult<SweepSummary> {
    Ok(store
        .sweep_summary(&SweepId::new(sweep.to_owned())?)
        .await?)
}
