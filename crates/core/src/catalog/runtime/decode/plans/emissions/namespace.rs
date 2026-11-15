use super::RuntimeCatalogError;
use crate::catalog::runtime::{ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan};

pub(super) fn validate(identity: &IdentityPlan, service: &str, operation: &str) -> Result<(), RuntimeCatalogError> {
    let fields = |fields: &[IdentityFieldPlan]| fields.iter().any(|field|
        matches!(field, IdentityFieldPlan::Context { service: producer_service, operation: producer_operation,
            source: ContextSource::SelectedItem, .. }
            if producer_service.as_ref() == service && producer_operation.as_ref() == operation));
    let leaf = |leaf: &IdentityLeafPlan| matches!(leaf, IdentityLeafPlan::Fields(values) if fields(values));
    let ambiguous = match identity {
        IdentityPlan::CorrelatedKey(values) => fields(values),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => leaf(fallback),
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches.iter().any(|branch| leaf(&branch.identity)),
        _ => false,
    };
    if ambiguous { Err(RuntimeCatalogError::Capability) } else { Ok(()) }
}

#[cfg(test)]
mod tests;
