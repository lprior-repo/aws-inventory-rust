//! Credential-free target and request accounting for offline plans.
use super::*;

pub(super) struct Targets {
    pub profiles: BTreeSet<String>,
    pub regions: BTreeSet<String>,
    pub profile_count: usize,
    pub target_count: Option<usize>,
}

pub(super) struct Requests {
    pub list: Option<usize>,
    pub root_list: Option<usize>,
    pub child: Value,
}

pub(super) fn targets(options: &PlanArgs) -> Targets {
    let profiles: BTreeSet<String> = options.profile.iter().cloned().collect();
    let regions: BTreeSet<String> = options.region.iter().cloned().collect();
    let profile_count = profiles.len().max(1);
    let target_count = if options.accounts.organization_role.is_some() {
        // Discovery can return unknown, suspended or inaccessible accounts;
        // no claimed target count is invented by this credential-free planner.
        None
    } else {
        Some(profile_count)
    };
    Targets {
        profiles,
        regions,
        profile_count,
        target_count,
    }
}

/// Universal mode runs one ListResources cursor per list-capable type and no
/// parent, child or GetResource stages; reviewed plan detail stays visible in
/// `coverage`, while request accounting reflects what the drain will do.
pub(super) fn request_accounting(catalog: &build::CatalogPlan, targets: &Targets) -> Requests {
    let (types, children) = match &catalog.universal {
        Some(universal) => (universal.list_scopes, 0_usize),
        None => (catalog.root_types, catalog.child_types),
    };
    let list = if targets.regions.is_empty() || children > 0 {
        None
    } else {
        bounded_requests(targets, types)
    };
    let root_list = if targets.regions.is_empty() {
        None
    } else {
        bounded_requests(targets, types)
    };
    Requests {
        list,
        root_list,
        child: if children > 0 { Value::Null } else { json!(0) },
    }
}

fn bounded_requests(targets: &Targets, types: usize) -> Option<usize> {
    let targets_count = targets.target_count?;
    types
        .checked_mul(targets_count)?
        .checked_mul(targets.regions.len())
}
