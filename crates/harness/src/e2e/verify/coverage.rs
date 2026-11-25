use super::coverage_status::Plans;
mod report;
use crate::{
    HarnessError,
    inventory_seed::{SeedError, command::directory, load_labeled},
};
use inventory_core::catalog::runtime::{ReviewDisposition, RuntimeCatalog};
use std::{collections::BTreeMap, path::Path};

pub struct Coverage {
    callable: Plans,
    deferred: BTreeMap<String, String>,
    steps: BTreeMap<String, String>,
}

impl Coverage {
    pub fn load(catalog: &Path, seeds: &Path, services: &[String]) -> Result<Self, HarnessError> {
        load(catalog, seeds, services)
            .map_err(|error| HarnessError::invalid(format!("verification catalog: {error}")))
    }

    pub(super) fn reason(&self, kind: &str) -> Result<Option<&str>, HarnessError> {
        if self.callable.contains_key(kind) {
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
    files.into_iter().try_fold(
        Coverage {
            callable,
            deferred: BTreeMap::new(),
            steps: BTreeMap::new(),
        },
        |mut coverage, path| {
            recipe_kinds(&catalog, &path, &mut coverage)?;
            Ok(coverage)
        },
    )
}

fn recipe_kinds(
    catalog: &RuntimeCatalog<'_>,
    path: &Path,
    coverage: &mut Coverage,
) -> Result<(), SeedError> {
    let bytes = directory::read_bounded(path, 1_048_576)?;
    let text =
        std::str::from_utf8(&bytes).map_err(|_| SeedError::Invalid("seed recipe is not UTF-8"))?;
    let (scenario, labels) = load_labeled(text)?;
    scenario
        .steps()
        .iter()
        .zip(labels)
        .try_for_each(|(step, label)| {
            let key = format!("{}\t{}\t{label}", step.service, step.operation);
            let kind = coverage
                .steps
                .entry(key)
                .or_insert_with(|| step.kind.clone());
            if *kind != step.kind {
                return Err(SeedError::Invalid(
                    "seed matrix label is ambiguous across recipes",
                ));
            }
            if !kind.is_empty() && !coverage.callable.contains_key(kind) {
                coverage
                    .deferred
                    .entry(kind.clone())
                    .or_insert_with(|| deferred_reason(catalog, &step.service));
            }
            Ok(())
        })
}

fn callable_kinds(catalog: &RuntimeCatalog<'_>) -> Plans {
    catalog
        .plans()
        .iter()
        .fold(Plans::new(), |mut kinds, plan| {
            let operation = format!("{}:{}", plan.service_name(), plan.operation_name());
            plan.resource_emissions()
                .iter()
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
                .for_each(|kind| {
                    kinds
                        .entry(kind.as_str().to_owned())
                        .or_default()
                        .insert(operation.clone());
                });
            kinds
        })
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
