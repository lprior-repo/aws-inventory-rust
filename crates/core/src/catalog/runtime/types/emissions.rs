use super::emission::{MAX_RESOURCE_EMISSIONS, ResourceEmissionPlan};
use super::super::RuntimeCatalogError;
use std::collections::BTreeSet;

/// The primary resource declaration and all additional selections, normalized once.
/// The primary discovery selector is stored separately on CallablePlan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEmissions {
    primary: ResourceEmissionPlan,
    additional: Box<[ResourceEmissionPlan]>,
}

impl ResourceEmissions {
    pub(crate) fn new(primary: ResourceEmissionPlan, additional: Vec<ResourceEmissionPlan>) -> Result<Self, RuntimeCatalogError> {
        if additional.len() >= MAX_RESOURCE_EMISSIONS { return Err(RuntimeCatalogError::Bounds); }
        if primary.key() != "primary" { return Err(RuntimeCatalogError::Capability); }
        let mut kinds = BTreeSet::new();
        let mut keys = BTreeSet::new();
        std::iter::once(&primary).chain(additional.iter()).try_for_each(|emission| {
            if emission.key().is_empty() || emission.key().len() > 64
                || !emission.key().bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
                || !keys.insert(emission.key()) { return Err(RuntimeCatalogError::Capability); }
            let candidates = std::iter::once(emission.kind()).chain(emission.classification()
                .into_iter().flat_map(|classification| classification.variants.iter().map(|(_, kind)| kind)))
                .collect::<BTreeSet<_>>();
            candidates.into_iter().try_for_each(|kind| {
                if kinds.insert(kind) { Ok(()) } else { Err(RuntimeCatalogError::Capability) }
            })
        })?;
        Ok(Self { primary, additional: additional.into_boxed_slice() })
    }

    pub fn primary(&self) -> &ResourceEmissionPlan { &self.primary }
    pub fn iter(&self) -> impl Iterator<Item = &ResourceEmissionPlan> {
        std::iter::once(&self.primary).chain(self.additional.iter())
    }
    pub fn len(&self) -> usize { self.additional.len().saturating_add(1) }
    pub fn is_empty(&self) -> bool { false }
}

#[cfg(test)]
mod tests;
