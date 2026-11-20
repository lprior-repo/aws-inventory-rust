use super::RuntimeCatalog;
use crate::inventory::Kind;

/// Kinds in the compiled catalog that no tag-bearing emission produces. Their resources
/// cannot receive derived owner/team/application values until a tag source is added.
pub fn kinds_without_tag_source(catalog: &RuntimeCatalog<'_>) -> Vec<Kind> {
    let plans = catalog.plans();
    catalog
        .schemas()
        .iter()
        .map(|schema| schema.kind())
        .filter(|kind| {
            !plans.iter().any(|plan| {
                plan.resource_emissions()
                    .iter()
                    .any(|emission| emission.tags().is_some() && emission.emits_kind(kind))
            })
        })
        .cloned()
        .collect()
}
