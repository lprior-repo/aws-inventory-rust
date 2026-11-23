use super::super::{PageCommit, SchemaRegistry, StoreError};
use inventory_core::inventory::{ResourceId, UnresolvedReference};
use std::collections::BTreeMap;

pub(in crate::inventory) fn references(
    registry: &SchemaRegistry,
    page: &PageCommit,
) -> Result<Vec<(UnresolvedReference, ResourceId)>, StoreError> {
    if page.edges.is_empty() {
        return Ok(Vec::new());
    }
    let observations = page
        .observations
        .iter()
        .map(|observation| (observation.key(), observation))
        .collect::<BTreeMap<_, _>>();
    let mut references = Vec::new();
    for edge in &page.edges {
        let source = observations
            .get(&edge.from)
            .ok_or(StoreError::Invalid("direct edge source evidence is absent"))?;
        let target = observations
            .get(&edge.to)
            .ok_or(StoreError::Invalid("direct edge target evidence is absent"))?;
        references.extend(
            super::super::relation_permissions::edge_references(
                registry,
                &page.lease,
                source,
                target,
                edge,
            )?
            .into_iter()
            .map(|reference| (reference, edge.to.id.clone())),
        );
    }
    Ok(references)
}
