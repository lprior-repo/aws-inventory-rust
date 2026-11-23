use super::*;
use inventory_core::catalog::runtime::ExecutionBranch;
pub(in crate::execution) async fn seed_enumerations(
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

pub(in crate::execution) const MAX_ROOT_SCOPES: usize = 100_000;

pub(in crate::execution) fn append_plan_work(
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
        append_branch(account, home, sweep, plan, regions, branch, work)?;
    }
    Ok(())
}

fn append_branch(
    account: &AccountId,
    home: &RegionId,
    sweep: &SweepId,
    plan: &CallablePlan<'_>,
    regions: &[Region],
    branch: Option<&ExecutionBranch>,
    work: &mut Vec<WorkSpec>,
) -> Result<(), SweepError> {
    if input::requires_parent_branch(plan, branch) {
        return Ok(());
    }
    let scope = branch.map_or(plan.scope(), |branch| branch.scope);
    for region in plan_regions(scope, regions, home) {
        if work.len() >= MAX_ROOT_SCOPES {
            return Err(SweepError::Operation(
                "root enumeration scope count exceeds bound",
            ));
        }
        let (scope, bindings) = root_input(account, home, plan, branch, region.scope)?;
        work.try_reserve(1)?;
        work.push(WorkSpec::new(sweep.clone(), scope, None, bindings)?);
    }
    Ok(())
}

fn root_input(
    account: &AccountId,
    home: &RegionId,
    plan: &CallablePlan<'_>,
    branch: Option<&ExecutionBranch>,
    region: Region,
) -> Result<(ScopeKey, Vec<u8>), SweepError> {
    let mut scope = scope_key(plan, account, region)?;
    if let Some(branch) = branch {
        scope = scope.with_branch(branch.discriminator.clone())?;
    }
    let request_region =
        super::super::routing::request_region(plan, scope.region(), home, scope.branch())?;
    let request_region = Region::Regional(request_region);
    let bindings = input::root_bindings_branch(plan, branch, Some(account), Some(&request_region))?
        .into_bytes();
    Ok((scope, bindings))
}

pub(in crate::execution) struct PlannedRegion {
    pub(in crate::execution) scope: Region,
    #[cfg(test)]
    pub(in crate::execution) request: Region,
}

#[cfg(test)]
fn planned_region(scope: Region, request: Region) -> PlannedRegion {
    PlannedRegion { scope, request }
}
#[cfg(not(test))]
fn planned_region(scope: Region, _request: Region) -> PlannedRegion {
    PlannedRegion { scope }
}

pub(in crate::execution) fn plan_regions(
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

pub(in crate::execution) fn scope_key(
    plan: &CallablePlan<'_>,
    account: &AccountId,
    region: Region,
) -> Result<ScopeKey, SweepError> {
    let service = ServiceId::parse(plan.service_name()).map_err(SweepError::from)?;
    let operation = OperationId::parse(plan.operation_name()).map_err(SweepError::from)?;
    ScopeKey::new(account.clone(), region, service, operation, None).map_err(SweepError::from)
}
