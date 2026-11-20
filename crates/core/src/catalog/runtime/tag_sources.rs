use super::{CallablePlan, PlanPurpose, RuntimeCatalog};
use crate::inventory::Kind;

/// Kinds in the compiled catalog that no tag source reaches. Their resources cannot
/// receive derived owner/team/application values until a tag source is added.
///
/// A kind is sourced when one of its own emissions carries tags, or when a per-resource
/// tag call (`PlanPurpose::TagEndpoint`) names a parent operation that emits the kind.
pub fn kinds_without_tag_source(catalog: &RuntimeCatalog<'_>) -> Vec<Kind> {
    let plans = catalog.plans();
    catalog
        .schemas()
        .iter()
        .map(|schema| schema.kind())
        .filter(|kind| !plans.iter().any(|plan| sources_kind(plans, plan, kind)))
        .cloned()
        .collect()
}

fn sources_kind(plans: &[CallablePlan<'_>], plan: &CallablePlan<'_>, kind: &Kind) -> bool {
    let own_tags = plan
        .resource_emissions()
        .iter()
        .any(|emission| emission.tags().is_some() && emission.emits_kind(kind));
    let via_parent = match plan.purpose() {
        PlanPurpose::TagEndpoint(endpoint) => endpoint
            .parents
            .iter()
            .any(|parent| parent_emits_kind(plans, plan.service_name(), &parent.operation, kind)),
        _ => false,
    };
    own_tags || via_parent
}

fn parent_emits_kind(
    plans: &[CallablePlan<'_>],
    service: &str,
    operation: &str,
    kind: &Kind,
) -> bool {
    plans.iter().any(|candidate| {
        candidate.service_name() == service
            && candidate.operation_name() == operation
            && candidate
                .resource_emissions()
                .iter()
                .any(|emission| emission.emits_kind(kind))
    })
}
