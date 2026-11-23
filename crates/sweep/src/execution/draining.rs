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

pub(super) async fn seed_enumerations(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    regions: &[Region],
) -> Result<u64, SweepError> {
    let work = enumeration_work(context, sweep, regions)?;
    let count = u64::try_from(work.len())
        .map_err(|_| SweepError::Operation("root scope count exceeds progress range"))?;
    context.store.seed_works(&work).await?;
    Ok(count)
}

fn enumeration_work(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    regions: &[Region],
) -> Result<Vec<WorkSpec>, SweepError> {
    let mut work = Vec::new();
    for plan in context
        .bootstrap
        .catalog
        .plans()
        .iter()
        .filter(|plan| matches!(plan.purpose(), PlanPurpose::Enumeration))
    {
        append_plan_work(
            &context.account,
            context.bootstrap.home,
            sweep,
            plan,
            regions,
            &mut work,
        )?;
    }
    Ok(work)
}

pub(super) const MAX_ROOT_SCOPES: usize = 100_000;

pub(super) fn append_plan_work(
    account: &AccountId,
    home: &RegionId,
    sweep: &SweepId,
    plan: &CallablePlan<'_>,
    regions: &[Region],
    work: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    for branch in plan
        .branches()
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| plan.branches().is_empty()))
    {
        if input::requires_parent_branch(plan, branch) {
            continue;
        }
        let scope_kind = branch.map_or(plan.scope(), |branch| branch.scope);
        for region in plan_regions(scope_kind, regions, home) {
            if work.len() >= MAX_ROOT_SCOPES {
                return Err(SweepError::Operation(
                    "root enumeration scope count exceeds bound",
                ));
            }
            let mut scope = scope_key(plan, account, region.scope)?;
            if let Some(branch) = branch {
                scope = scope.with_branch(branch.discriminator.clone())?;
            }
            let request_region =
                super::routing::request_region(plan, scope.region(), home, scope.branch())?;
            let request_region = Region::Regional(request_region);
            let bindings =
                input::root_bindings_branch(plan, branch, Some(account), Some(&request_region))?
                    .into_bytes();
            work.try_reserve(1)?;
            work.push(WorkSpec::new(sweep.clone(), scope, None, bindings)?);
        }
    }
    Ok(())
}

pub(super) struct PlannedRegion {
    pub(super) scope: Region,
    #[cfg(test)]
    pub(super) request: Region,
}

#[cfg(test)]
fn planned_region(scope: Region, request: Region) -> PlannedRegion {
    PlannedRegion { scope, request }
}
#[cfg(not(test))]
fn planned_region(scope: Region, _request: Region) -> PlannedRegion {
    PlannedRegion { scope }
}

pub(super) fn plan_regions(
    scope: PlanScope,
    regions: &[Region],
    home: &RegionId,
) -> Vec<PlannedRegion> {
    match scope {
        PlanScope::Region => regions
            .iter()
            .cloned()
            .map(|region| planned_region(region.clone(), region))
            .collect(),
        PlanScope::Account => vec![planned_region(
            Region::Global,
            Region::Regional(home.clone()),
        )],
        PlanScope::HomeRegion => vec![planned_region(
            Region::Regional(home.clone()),
            Region::Regional(home.clone()),
        )],
    }
}

pub(super) fn scope_key(
    plan: &CallablePlan<'_>,
    account: &AccountId,
    region: Region,
) -> Result<ScopeKey, SweepError> {
    let service = ServiceId::parse(plan.service_name()).map_err(SweepError::from)?;
    let operation = OperationId::parse(plan.operation_name()).map_err(SweepError::from)?;
    ScopeKey::new(account.clone(), region, service, operation, None).map_err(SweepError::from)
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
    if stored.work_pending != 0 {
        // Only a slice that ran out of time may leave work behind; the next
        // invocation resumes it. Any other pending work is an engine fault.
        if !slice_exhausted {
            return Err(SweepError::Operation(
                "sweep ended with active work; incomplete completion is not reportable",
            ));
        }
        summary.work_remaining = stored.work_pending;
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
