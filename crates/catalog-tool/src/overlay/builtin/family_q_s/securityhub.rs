// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{NameError, ServiceOverlay, construction};
use super::super::{name, path};
use crate::overlay::{ExecutionBranch, FixedValue, Identity, IdentityField, Scope};
use std::collections::BTreeMap;
mod calls_0;
mod calls_1;
mod decisions_0;
mod decisions_1;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut service = construction::reviewed(
        "securityhub",
        &[calls_0::CALLS, calls_1::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS],
    )?;
    if let Some(plan) = service.operations.get_mut(&name("ListOrganizationAdminAccounts")?) {
        plan.branches = feature_branches()?;
        plan.identity = Some(Identity::CorrelatedKey(vec![
            IdentityField::Request(path("Feature")?),
            IdentityField::Selected(path("AccountId")?),
        ]));
    }
    Ok(service)
}

fn feature_branches() -> Result<Vec<ExecutionBranch>, NameError> {
    ["SecurityHub", "SecurityHubV2"].into_iter().map(|feature| {
        Ok(ExecutionBranch {
            discriminator: feature.to_owned(),
            fixed: BTreeMap::from([(path("Feature")?, FixedValue::String(feature.to_owned()))]),
            bind: BTreeMap::new(),
            emission: None,
            row_policy: None,
            scope: Scope::Region,
            request_region: None,
        })
    }).collect()
}
