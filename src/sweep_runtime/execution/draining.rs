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
    context: &RuntimeContext<'_, '_>,
    sweep: &SweepId,
    regions: &[Region],
) -> Result<(), SweepError> {
    let mut work = Vec::new();
    for plan in context
        .bootstrap
        .catalog
        .plans()
        .iter()
        .filter(|plan| matches!(plan.purpose(), PlanPurpose::Enumeration))
    {
        if !input::requires_parent(plan) {
            append_plan_work(context, sweep, plan, regions, &mut work)?;
        }
    }
    context.store.seed_works(&work).await?;
    Ok(())
}

pub(super) const MAX_ROOT_SCOPES: usize = 100_000;

pub(super) fn append_plan_work(
    context: &RuntimeContext<'_, '_>,
    sweep: &SweepId,
    plan: &CallablePlan<'_>,
    regions: &[Region],
    work: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    for region in plan_regions(plan.scope(), regions, context.bootstrap.home) {
        if work.len() >= MAX_ROOT_SCOPES {
            return Err(SweepError::Operation(
                "root enumeration scope count exceeds bound",
            ));
        }
        work.try_reserve(1)?;
        let scope = scope_key(plan, &context.account, region.scope)?;
        let bindings =
            input::root_bindings(plan, Some(&context.account), Some(&region.request))?.into_bytes();
        work.push(WorkSpec::new(sweep.clone(), scope, None, bindings)?);
    }
    Ok(())
}

pub(super) struct PlannedRegion {
    pub(super) scope: Region,
    pub(super) request: Region,
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
            .map(|region| PlannedRegion {
                scope: region.clone(),
                request: region,
            })
            .collect(),
        PlanScope::Account => vec![PlannedRegion {
            scope: Region::Global,
            request: Region::Regional(home.clone()),
        }],
        PlanScope::HomeRegion => vec![PlannedRegion {
            scope: Region::Regional(home.clone()),
            request: Region::Regional(home.clone()),
        }],
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
    context: &RuntimeContext<'_, '_>,
    sweep: &SweepId,
) -> Result<SweepSummary, SweepError> {
    let mut summary = SweepSummary::default();
    loop {
        let mut leases = Vec::new();
        while leases.len() < context.bootstrap.config.concurrency {
            let lease = context
                .store
                .claim(sweep, now_ms()?, context.lease_ms)
                .await?;
            let Some(lease) = lease else {
                break;
            };
            leases.try_reserve(1)?;
            leases.push(lease);
        }
        if leases.is_empty() {
            break;
        }
        let work = stream::iter(
            leases
                .into_iter()
                .map(|lease| page::process(context, lease)),
        )
        .buffer_unordered(context.bootstrap.config.concurrency)
        .collect::<Vec<_>>();
        let results = work.await;
        for result in results {
            add_writes(&mut summary, result?)?;
        }
    }
    apply_persisted_summary(context.store, sweep, &mut summary).await?;
    summary.graph_pending = context.store.projection_state().await?.pending;
    Ok(summary)
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
    store: &BinaryStore,
    sweep: &SweepId,
    summary: &mut SweepSummary,
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
        return Err(SweepError::Operation(
            "sweep ended with active work; incomplete completion is not reportable",
        ));
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
