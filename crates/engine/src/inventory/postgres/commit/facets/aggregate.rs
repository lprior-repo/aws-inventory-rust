use super::Facet;
use crate::inventory::facets::{Aggregate, exclusions, merge, relations};
use crate::inventory::relation_permissions::ReferenceProvenance;
use crate::inventory::{SchemaRegistry, StoreError};
use inventory_core::inventory::{Observation, ResourceId, ResourceKey, UnresolvedReference};
use inventory_core::record::{Exclusions, FieldId, MapKeyExclusion};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Reference = (UnresolvedReference, Option<ResourceId>, ReferenceProvenance);

#[derive(Default)]
struct Overlay {
    fields: merge::Fields,
    owners: BTreeMap<merge::Path, usize>,
    exclusions: Vec<Vec<FieldId>>,
    map_exclusions: BTreeSet<MapKeyExclusion>,
    name: Option<Arc<str>>,
}

pub(in crate::inventory::postgres) fn build(
    registry: &SchemaRegistry,
    key: &ResourceKey,
    facets: &mut [Facet<'_>],
) -> Result<Aggregate, StoreError> {
    facets.sort_unstable_by(|left, right| {
        (left.precedence, &left.identity).cmp(&(right.precedence, &right.identity))
    });
    let mut overlay = Overlay::default();
    for (index, facet) in facets.iter().enumerate() {
        overlay.append(facet, index)?;
    }
    let references = references(registry, key, facets, &overlay.owners)?;
    Ok(Aggregate {
        observation: overlay.observation(key)?,
        references,
        relations_changed: false,
        record_changed: false,
        bound_context_changed: false,
    })
}

impl Overlay {
    fn append(&mut self, facet: &Facet<'_>, index: usize) -> Result<(), StoreError> {
        merge::overlay(
            &mut self.fields,
            &mut self.owners,
            &facet.owned,
            merge::fields(&facet.record)?,
            index,
        )?;
        exclusions::append(
            &facet.exclusions,
            &mut self.exclusions,
            &mut self.map_exclusions,
        )?;
        if facet.name_owned {
            self.name = facet.name.clone();
        }
        Ok(())
    }

    fn observation(self, key: &ResourceKey) -> Result<Observation, StoreError> {
        let masks = Exclusions::new(self.exclusions)?
            .with_map_keys(self.map_exclusions.into_iter().collect())?;
        Ok(Observation::new(
            key.clone(),
            merge::record(self.fields)?,
            &masks,
            self.name,
        )?)
    }
}

fn references(
    registry: &SchemaRegistry,
    key: &ResourceKey,
    facets: &[Facet<'_>],
    owners: &BTreeMap<merge::Path, usize>,
) -> Result<Vec<Reference>, StoreError> {
    let mut references = Vec::new();
    for (index, facet) in facets.iter().enumerate() {
        references.extend(relations::decode(
            registry,
            &facet.relations,
            key,
            index,
            owners,
            &facet.identity.source,
            facet.source_authority.as_ref(),
        )?);
    }
    Ok(references)
}
