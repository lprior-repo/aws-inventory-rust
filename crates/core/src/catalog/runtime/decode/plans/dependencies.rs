use super::{CallablePlan, RuntimeCatalogError};
use crate::catalog::runtime::{Emission, IdentityChoicePlan, IdentityLeafPlan, PlanPurpose};

pub(super) fn validate(plans: &mut [CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    super::tag_endpoint::validate_parents(plans)?;
    super::routing::validate_plans(plans)?;
    super::context::validate(plans)?;
    super::row_policy::validate_parents(plans)?;
    plans.iter().enumerate().filter(|(_, plan)| resource(plan)).try_for_each(|(index, plan)| {
        plans.iter().take(index).filter(|other| resource(other) && other.kind() == plan.kind())
            .try_for_each(|other| {
                if same_domain(plan.identity(), other.identity()) { Ok(()) }
                else { Err(RuntimeCatalogError::Capability) }
            })
    })?;
    let depths = super::stages::depths(plans)?;
    plans.iter_mut().zip(depths).try_for_each(|(plan, depth)| {
        plan.set_facet_precedence(depth);
        Ok(())
    })
}

fn resource(plan: &CallablePlan<'_>) -> bool {
    matches!(plan.purpose(), PlanPurpose::Enumeration) && plan.emission() == Emission::EmitMetadata
}


fn same_domain(left: &super::IdentityPlan, right: &super::IdentityPlan) -> bool {
    if left == right { return true; }
    arn_domain(left) && arn_domain(right)
}

fn arn_domain(identity: &super::IdentityPlan) -> bool {
    match identity {
        super::IdentityPlan::Arn(_) => true,
        super::IdentityPlan::Template(parts) => {
            matches!(parts.first(), Some(super::IdentityPart::Literal(prefix)) if prefix.starts_with("arn:"))
                && parts.iter().filter_map(|part| match part {
                    super::IdentityPart::Literal(value) => Some(value.bytes().filter(|byte| *byte == b':').count()),
                    _ => None,
                }).sum::<usize>() >= 5
        }
        super::IdentityPlan::MapArn { .. } => true,
        super::IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => leaf_arn(fallback),
        super::IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches.iter().all(|branch| leaf_arn(&branch.identity)),
        super::IdentityPlan::Key(_) | super::IdentityPlan::CorrelatedKey(_) => false,
    }
}

fn leaf_arn(identity: &IdentityLeafPlan) -> bool {
    match identity {
        IdentityLeafPlan::Arn(_) => true,
        IdentityLeafPlan::Template(parts) => {
            matches!(parts.first(), Some(super::IdentityPart::Literal(prefix)) if prefix.starts_with("arn:"))
                && parts.iter().filter_map(|part| match part {
                    super::IdentityPart::Literal(value) => Some(value.bytes().filter(|byte| *byte == b':').count()),
                    _ => None,
                }).sum::<usize>() >= 5
        }
        _ => false,
    }
}
