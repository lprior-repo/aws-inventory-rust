use super::{Args, CloudControlError};
use crate::{HarnessError, inventory_seed::command::directory};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) struct Target {
    pub type_name: String,
    pub kind: String,
    pub listable: bool,
}

pub(super) struct Inputs {
    pub targets: Vec<Target>,
    pub seeded: BTreeSet<String>,
    pub no_cfn: BTreeSet<String>,
    pub explanations: String,
    pub pem: Vec<u8>,
}

pub(super) fn read(args: &Args) -> Result<Inputs, HarnessError> {
    let bytes = bounded(&args.catalog, 67_108_864)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes).map_err(message)?;
    let universal: Value = serde_json::from_slice(&bounded(&args.universal, 2_097_152)?)?;
    let capabilities = capabilities(&universal)?;
    let targets = catalog
        .coverage()
        .iter()
        .filter_map(|coverage| coverage.cfn_type.as_ref().zip(coverage.kind.as_ref()))
        .map(|(type_name, kind)| {
            let listable = capabilities.get(type_name).copied().ok_or_else(|| {
                HarnessError::invalid(format!(
                    "Cloud Control type missing from universal catalog: {type_name}"
                ))
            })?;
            Ok(Target {
                type_name: type_name.clone(),
                kind: kind.as_str().to_owned(),
                listable,
            })
        })
        .collect::<Result<Vec<_>, HarnessError>>()?;
    let (seeded, no_cfn) = seed_gate(&catalog, seeded(args)?, &targets)?;
    if args.differences.is_dir() {
        return Err(HarnessError::invalid(format!(
            "Cloud Control differences input is a directory: {}",
            args.differences.display()
        )));
    }
    let explanations = match bounded(&args.differences, 1_048_576) {
        Ok(bytes) => String::from_utf8(bytes).map_err(message)?,
        Err(HarnessError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(HarnessError::invalid(format!(
                "Cloud Control differences input not found: {}",
                args.differences.display()
            )));
        }
        Err(error) => return Err(error),
    };
    Ok(Inputs {
        targets,
        seeded,
        no_cfn,
        explanations,
        pem: bounded(&args.ca_bundle, 1_048_576)?,
    })
}

fn seed_gate(
    catalog: &RuntimeCatalog<'_>,
    mut seeded: BTreeSet<String>,
    targets: &[Target],
) -> Result<(BTreeSet<String>, BTreeSet<String>), HarnessError> {
    let cfn: BTreeSet<_> = catalog
        .coverage()
        .iter()
        .filter(|coverage| coverage.cfn_type.is_some())
        .flat_map(|coverage| {
            coverage
                .cfn_type
                .as_deref()
                .into_iter()
                .chain(coverage.kind.as_ref().map(|kind| kind.as_str()))
        })
        .collect();
    let known: BTreeSet<_> = catalog
        .schemas()
        .iter()
        .map(|schema| schema.kind().as_str())
        .chain(
            catalog
                .coverage()
                .iter()
                .filter_map(|coverage| coverage.kind.as_ref().map(|kind| kind.as_str())),
        )
        .chain(cfn.iter().copied())
        .collect();
    let kinds: Vec<_> = seeded
        .extract_if(.., |kind| !known.contains(kind.as_str()))
        .collect();
    if !kinds.is_empty() {
        return Err(CloudControlError::UnknownKinds { kinds }.into());
    }
    let (mut gated, excluded): (BTreeSet<_>, BTreeSet<_>) = seeded
        .into_iter()
        .partition(|kind| cfn.contains(kind.as_str()));
    let kinds: Vec<_> = gated
        .extract_if(.., |kind| {
            !targets
                .iter()
                .any(|target| target.kind == *kind || target.type_name == *kind)
        })
        .collect();
    if !kinds.is_empty() {
        return Err(CloudControlError::UnmappedKinds { kinds }.into());
    }
    Ok((gated, excluded))
}

fn capabilities(document: &Value) -> Result<BTreeMap<String, bool>, HarnessError> {
    if document.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(HarnessError::invalid(
            "unsupported Cloud Control universal catalog version",
        ));
    }
    let rows = document
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            HarnessError::invalid("Cloud Control universal catalog lacks resource_types")
        })?;
    rows.iter().try_fold(BTreeMap::new(), |mut types, row| {
        let name = row
            .get("type_name")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessError::invalid("Cloud Control universal type lacks type_name"))?;
        let listable = match row.get("capability").and_then(Value::as_str) {
            Some("list") => true,
            Some("parent" | "unsupported") => false,
            _ => {
                return Err(HarnessError::invalid(
                    "unknown Cloud Control list capability",
                ));
            }
        };
        if types.insert(name.to_owned(), listable).is_some() {
            return Err(HarnessError::invalid(
                "duplicate Cloud Control universal type",
            ));
        }
        Ok(types)
    })
}

fn seeded(args: &Args) -> Result<BTreeSet<String>, HarnessError> {
    if let Some(oracle) = &args.oracle {
        let text = String::from_utf8(bounded(oracle, 16_777_216)?).map_err(message)?;
        return oracle_kinds(&text);
    }
    directory::files(&args.seeds)
        .map_err(message)?
        .into_iter()
        .try_fold(BTreeSet::new(), |kinds, path| {
            let scenario = directory::read_scenario(&path).map_err(message)?;
            Ok(scenario
                .steps()
                .iter()
                .filter(|step| !step.kind.is_empty())
                .fold(kinds, |mut kinds, step| {
                    kinds.insert(step.kind.clone());
                    kinds
                }))
        })
}

fn oracle_kinds(text: &str) -> Result<BTreeSet<String>, HarnessError> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .try_fold(BTreeSet::new(), |mut kinds, line| {
            let columns: Vec<_> = line.split('\t').collect();
            match columns.first().copied() {
                Some("resource") if columns.len() == 5 => {
                    let kind = columns
                        .get(1)
                        .filter(|kind| !kind.is_empty())
                        .ok_or_else(|| HarnessError::invalid("seed oracle resource lacks kind"))?;
                    kinds.insert((*kind).to_owned());
                }
                Some("edge") if columns.len() == 6 => {}
                _ => {
                    return Err(HarnessError::invalid(
                        "invalid Cloud Control seed oracle row",
                    ));
                }
            }
            Ok(kinds)
        })
}

pub(super) fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>, HarnessError> {
    let limit = usize::try_from(limit).map_err(message)?;
    crate::files::read_limited(std::fs::File::open(path)?, limit).map_err(|error| match error {
        crate::files::ReadError::Io(error) => HarnessError::Io(error),
        crate::files::ReadError::TooLarge => {
            HarnessError::invalid("Cloud Control input byte bound exceeded")
        }
        crate::files::ReadError::InvalidBudget => {
            HarnessError::invalid("Cloud Control input byte bound is not representable")
        }
    })
}

fn message(error: impl std::fmt::Display) -> HarnessError {
    HarnessError::invalid(error.to_string())
}

#[cfg(test)]
mod tests;
