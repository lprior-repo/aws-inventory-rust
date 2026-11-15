use super::{CallablePlan, RuntimeCatalogError, optional_target, scalar_matches, tags};
use crate::{catalog::runtime::{RelationSource, RelationshipPlan, ResourceEmissionPlan, ResourceProjectionPlan}, inventory::ScalarType};

impl CallablePlan<'_> {
    /// Validate the exact resource namespace; execution-stage admission is the caller's job.
    /// A descriptor from another operation cannot borrow this operation's modeled authority.
    pub fn supports_emission_relationship(&self, emission: &ResourceEmissionPlan,
        relation: &RelationshipPlan,
    ) -> Result<bool, RuntimeCatalogError> {
        if !self.resource_emissions().iter().any(|candidate| std::ptr::eq(candidate, emission))
            || !emission.emits_kind(&relation.source_kind) { return Ok(false); }
        let target = match relation.source {
            RelationSource::SelectedOutput => {
                if matches!(emission.projection(), Some(ResourceProjectionPlan::IamTrustSummary { path })
                    if path.as_ref() == relation.source_path.as_ref()) {
                    return Err(RuntimeCatalogError::Capability);
                }
                tags::selected_item(self.service(), self.operation(), emission.items())?
            }
            RelationSource::Request => self.operation().input()?,
        };
        match optional_target(self.service(), target, &relation.source_path)? {
            None => Ok(false),
            Some(target) if scalar_matches(tags::target_kind(self.service(), target)?, ScalarType::Text) => Ok(true),
            Some(_) => Err(RuntimeCatalogError::Capability),
        }
    }
}
