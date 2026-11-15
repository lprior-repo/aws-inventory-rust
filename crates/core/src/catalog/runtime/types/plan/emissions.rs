use super::{CallablePlan, Emission, ExecutionBranch};
use crate::catalog::runtime::ResourceEmissions;

impl CallablePlan<'_> {
    pub fn resource_emissions(&self) -> &ResourceEmissions { &self.parts.resources }

    /// The branch must first be resolved from this plan and the durable work scope.
    /// Discovery-only stages cannot admit any metadata, including sibling selectors.
    pub fn effective_resource_emissions(&self, branch: Option<&ExecutionBranch>) -> Option<&ResourceEmissions> {
        (self.effective_emission(branch) == Emission::EmitMetadata).then_some(self.resource_emissions())
    }
}
