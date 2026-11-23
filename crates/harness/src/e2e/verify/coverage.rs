use crate::{
    HarnessError,
    inventory_seed::{self, SeedError, command::directory},
};
use inventory_core::catalog::runtime::{ReviewDisposition, RuntimeCatalog};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub struct Coverage {
    callable: BTreeSet<String>,
    deferred: BTreeMap<String, String>,
}

impl Coverage {
    pub fn load(catalog: &Path, seeds: &Path, services: &[String]) -> Result<Self, HarnessError> {
        load(catalog, seeds, services)
            .map_err(|error| HarnessError::invalid(format!("verification catalog: {error}")))
    }

    pub(super) fn reason(&self, kind: &str) -> Result<Option<&str>, HarnessError> {
        if self.callable.contains(kind) {
            return Ok(None);
        }
        self.deferred
            .get(kind)
            .map(|reason| Some(reason.as_str()))
            .ok_or_else(|| HarnessError::invalid(format!("no recipe/catalog evidence for {kind}")))
    }
}

fn load(catalog: &Path, seeds: &Path, services: &[String]) -> Result<Coverage, SeedError> {
    let bytes = directory::read_bounded(catalog, 67_108_864)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let callable = callable_kinds(&catalog);
    let files = directory::select(directory::files(seeds)?, services)?;
    let deferred = files.into_iter().try_fold(BTreeMap::new(), |kinds, path| {
        recipe_kinds(&catalog, &path, &callable, kinds)
    })?;
    Ok(Coverage { callable, deferred })
}

fn recipe_kinds(
    catalog: &RuntimeCatalog<'_>,
    path: &Path,
    callable: &BTreeSet<String>,
    kinds: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, SeedError> {
    let text = directory::read_bounded(path, 1_048_576)?;
    let text =
        std::str::from_utf8(&text).map_err(|_| SeedError::Invalid("seed recipe is not UTF-8"))?;
    Ok(inventory_seed::load(text)?
        .steps()
        .iter()
        .filter(|step| !step.kind.is_empty() && !callable.contains(&step.kind))
        .fold(kinds, |mut kinds, step| {
            if !kinds.contains_key(&step.kind) {
                kinds.insert(step.kind.clone(), deferred_reason(catalog, &step.service));
            }
            kinds
        }))
}

fn callable_kinds(catalog: &RuntimeCatalog<'_>) -> BTreeSet<String> {
    catalog
        .plans()
        .iter()
        .flat_map(|plan| plan.resource_emissions().iter())
        .flat_map(|emission| {
            std::iter::once(emission.kind()).chain(
                emission
                    .classification()
                    .into_iter()
                    .flat_map(|classification| {
                        classification.variants.iter().map(|(_, kind)| kind)
                    }),
            )
        })
        .map(|kind| kind.as_str().to_owned())
        .collect()
}

fn deferred_reason(catalog: &RuntimeCatalog<'_>, service: &str) -> String {
    let reasons = catalog
        .decisions()
        .iter()
        .filter(|decision| decision.service == service)
        .filter_map(|decision| match &decision.disposition {
            ReviewDisposition::Deferred { reason } => Some(format!(
                "{}:{}: {reason}",
                decision.service, decision.operation
            )),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("; ");
    if reasons.is_empty() {
        return "no callable resource emission and no service-level deferred review decision"
            .into();
    }
    format!("no callable resource emission; service-level overlay review decisions: {reasons}")
}

#[cfg(test)]
mod tests;
