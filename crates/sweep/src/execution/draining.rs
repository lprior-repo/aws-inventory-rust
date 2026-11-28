#[path = "draining/seeding.rs"]
mod seeding;
pub(super) use seeding::*;
#[path = "draining/admission.rs"]
pub(super) mod admission;
#[path = "draining/settlement.rs"]
mod settlement;
pub(super) use settlement::settle_batch;
use settlement::settle_owned;

use super::*;

pub(super) fn narrow_regions(
    regions: Vec<Region>,
    requested: Option<RegionId>,
) -> Result<Vec<Region>, SweepError> {
    match requested {
        Some(requested) => regions
            .into_iter()
            .find(|region| region.as_str() == requested.as_str())
            .map(|region| vec![region])
            .ok_or(SweepError::Operation(
                "requested region is not returned by the reviewed enabled-region plan",
            )),
        None => Ok(regions),
    }
}

pub(super) async fn drain_work(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    budget: SliceBudget,
) -> Result<SweepSummary, SweepError> {
    let executor = context.bootstrap.executor;
    let owned = admission::Owned::default();
    let slice_exhausted = std::sync::atomic::AtomicBool::new(false);
    let work = scheduler::run(
        context.bootstrap.config.concurrency,
        || may_claim(executor, budget, &slice_exhausted),
        |limit| claim(context, sweep, &owned, limit),
        |lease| {
            let id = owned.admit(&lease);
            owned.settle(
                id,
                lease,
                || may_claim(executor, budget, &slice_exhausted),
                |lease, keep| page::process(context, lease, keep),
            )
        },
        |in_flight| owned.set_in_flight(in_flight),
    );
    let mut summary = settle_batch(context.store, executor, || owned.take(), work).await?;
    if context.bootstrap.executor.is_cancelled() {
        return Err(inventory_engine::limiter::ApiError::Cancelled.into());
    }
    let slice_exhausted = slice_exhausted.load(std::sync::atomic::Ordering::Relaxed);
    apply_persisted_summary(context.store, sweep, &mut summary, slice_exhausted).await?;
    summary.graph_pending = context.store.projection_state().await?.pending;
    Ok(summary)
}

fn may_claim(
    executor: &ApiExecutor,
    budget: SliceBudget,
    slice_exhausted: &std::sync::atomic::AtomicBool,
) -> bool {
    use std::sync::atomic::Ordering::Relaxed;
    if slice_exhausted.load(Relaxed) || executor.is_cancelled() {
        return false;
    }
    let Ok(now) = now_ms() else { return true };
    let exhausted = budget.exhausted(now);
    slice_exhausted.store(exhausted, Relaxed);
    !exhausted
}

async fn claim(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    owned: &admission::Owned,
    limit: u16,
) -> Result<Vec<inventory_engine::inventory::WorkLease>, SweepError> {
    let leases = admission::claim_some(
        context.store,
        sweep,
        context.lease_ms,
        limit,
        context.progress.enabled(),
    )
    .await?;
    context.bootstrap.metrics.claimed(leases.len());
    let (depth, in_flight) = admission::queue_report(
        leases.first().and_then(|lease| lease.pending_work()),
        owned.in_flight(),
        leases.len(),
    );
    context.progress.queue(depth, in_flight);
    Ok(leases)
}

pub(super) fn add_writes(
    summary: &mut SweepSummary,
    other: SweepSummary,
) -> Result<(), SweepError> {
    summary.resource_writes = summary
        .resource_writes
        .checked_add(other.resource_writes)
        .ok_or(SweepError::Operation("resource counter overflow"))?;
    Ok(())
}

pub(super) async fn apply_persisted_summary(
    store: &impl InventoryStore,
    sweep: &SweepId,
    summary: &mut SweepSummary,
    slice_exhausted: bool,
) -> Result<(), SweepError> {
    let stored = store.sweep_summary(sweep).await?;
    summary.rows = stored.resources_observed;
    summary.outcomes = ScopeOutcomes {
        complete: stored.complete_scopes,
        partial: stored.partial_scopes,
        inaccessible: stored.inaccessible_scopes,
        unsupported: stored.unsupported_scopes,
        not_applicable: stored.not_applicable_scopes,
        failed: stored.failed_scopes,
        unobserved: stored.unobserved_scopes,
    };
    summary.scopes = scope_total(&summary.outcomes)?;
    record_pending(summary, stored.work_pending, slice_exhausted)?;
    Ok(())
}

fn record_pending(
    summary: &mut SweepSummary,
    pending: u64,
    slice_exhausted: bool,
) -> Result<(), SweepError> {
    if pending != 0 {
        if !slice_exhausted {
            return Err(SweepError::Operation(
                "sweep ended with active work; incomplete completion is not reportable",
            ));
        }
        summary.work_remaining = pending;
    }
    Ok(())
}

pub(super) fn scope_total(outcomes: &ScopeOutcomes) -> Result<u64, SweepError> {
    [
        outcomes.complete,
        outcomes.partial,
        outcomes.inaccessible,
        outcomes.unsupported,
        outcomes.not_applicable,
        outcomes.failed,
        outcomes.unobserved,
    ]
    .into_iter()
    .try_fold(0_u64, |total, count| {
        total
            .checked_add(count)
            .ok_or(SweepError::Operation("persisted scope count overflow"))
    })
}

pub(super) fn now_ms() -> Result<i64, SweepError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SweepError::Operation("system clock precedes Unix epoch"))?;
    i64::try_from(duration.as_millis())
        .map_err(|_| SweepError::Operation("system time exceeds timestamp bound"))
}
