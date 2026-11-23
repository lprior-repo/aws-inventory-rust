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
    let mut summary = SweepSummary::default();
    let mut leases = Vec::new();
    let mut slice_exhausted;
    loop {
        slice_exhausted = budget.exhausted(now_ms()?);
        if slice_exhausted || !drain_batch(context, sweep, &mut leases, &mut summary).await? {
            break;
        }
    }
    if context.bootstrap.executor.is_cancelled() {
        return Err(inventory_engine::limiter::ApiError::Cancelled.into());
    }
    apply_persisted_summary(context.store, sweep, &mut summary, slice_exhausted).await?;
    summary.graph_pending = context.store.projection_state().await?.pending;
    Ok(summary)
}

async fn drain_batch(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    leases: &mut Vec<inventory_engine::inventory::WorkLease>,
    summary: &mut SweepSummary,
) -> Result<bool, SweepError> {
    claim_batch(context, sweep, leases).await?;
    if leases.is_empty() {
        return Ok(false);
    }
    let owned = leases.clone();
    let work = stream::iter(leases.drain(..).map(|lease| page::process(context, lease)))
        .buffer_unordered(context.bootstrap.config.concurrency);
    *summary = settle_batch(
        context.store,
        context.bootstrap.executor,
        owned,
        summarize_batch(work, *summary),
    )
    .await?;
    Ok(true)
}

async fn claim_batch(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    leases: &mut Vec<inventory_engine::inventory::WorkLease>,
) -> Result<(), SweepError> {
    admission::claim_batch(
        context.store,
        context.bootstrap.executor,
        context.bootstrap.config.concurrency,
        leases,
        context.progress,
        || admission::claim_next(context, sweep),
    )
    .await
}

async fn summarize_batch(
    results: impl futures_util::Stream<Item = Result<SweepSummary, SweepError>>,
    summary: SweepSummary,
) -> Result<SweepSummary, SweepError> {
    // Unlike try_fold, fold keeps polling every admitted page after an error.
    // Preserve the first completion-order error without retaining all summaries.
    results
        .fold(Ok(summary), |summary, result| async move {
            let mut summary = summary?;
            add_writes(&mut summary, result?)?;
            Ok(summary)
        })
        .await
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

#[cfg(test)]
#[path = "draining/tests.rs"]
mod tests;
