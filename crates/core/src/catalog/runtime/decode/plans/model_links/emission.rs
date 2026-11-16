use super::{CallablePlan, RuntimeCatalogError, optional_target, relationship_text, tags};
use crate::{catalog::runtime::{RelationSource, RelationshipPlan, ResourceEmissionPlan, ResourceProjectionPlan}, record::FieldId};

impl CallablePlan<'_> {
    /// Validate the exact resource namespace; execution-stage admission is the caller's job.
    /// A descriptor from another operation cannot borrow this operation's modeled authority.
    pub fn supports_emission_relationship(&self, emission: &ResourceEmissionPlan,
        relation: &RelationshipPlan,
    ) -> Result<bool, RuntimeCatalogError> {
        if !self.resource_emissions().iter().any(|candidate| std::ptr::eq(candidate, emission))
            || !emission.emits_kind(&relation.source_kind) { return Ok(false); }
        let supported = supports(self, emission, relation.source, &relation.source_path)?;
        if !supported { return Ok(false); }
        match &relation.guard {
            None => Ok(true),
            Some(guard) => supports(self, emission, guard.source, &guard.path),
        }
    }
}

fn supports(plan: &CallablePlan<'_>, emission: &ResourceEmissionPlan, source: RelationSource,
    path: &[FieldId],
) -> Result<bool, RuntimeCatalogError> {
    let target = match source {
        RelationSource::SelectedOutput => {
            if matches!(emission.projection(), Some(ResourceProjectionPlan::IamTrustSummary { path: protected })
                if protected.as_ref() == path) {
                return Err(RuntimeCatalogError::Capability);
            }
            tags::selected_item(plan.service(), plan.operation(), emission.items())?
        }
        RelationSource::Request => plan.operation().input()?,
    };
    match optional_target(plan.service(), target, path)? {
        None => Ok(false),
        Some(target) if relationship_text(plan.service(), target)? => Ok(true),
        Some(_) => Err(RuntimeCatalogError::Capability),
    }
}
