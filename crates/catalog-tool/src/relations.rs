mod model;
mod pointer;
mod remap;
#[cfg(test)]
mod tests;

use crate::{ReportMode, kind_map, overlay, report, smithy};
use serde::Deserialize;
use std::{io::Write as _, path::Path};

pub const ROWS_PATH: &str = "catalog/relations.generated.tsv";
pub const REPORT_PATH: &str = "catalog/aws-relations-remap.report";
const MAX_RULES: usize = 10_000;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Report(#[from] report::ReportError),
    #[error(transparent)]
    Kinds(#[from] kind_map::KindMapError),
    #[error(transparent)]
    Model(#[from] crate::runtime_catalog::RuntimeCatalogCompileError),
    #[error(transparent)]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error(transparent)]
    Read(#[from] smithy::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Review(Box<overlay::CheckError>),
    #[error("invalid relation input: {0}")]
    Invalid(&'static str),
}

#[derive(Deserialize)]
struct LegacyCatalog {
    resource_types: Vec<LegacyType>,
}

#[derive(Deserialize)]
struct LegacyType {
    type_name: String,
    #[serde(default)]
    kind: String,
    view_name: String,
    #[serde(default)]
    relationships: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    pointer: String,
    relation: String,
    target_kind: String,
    target_format: String,
}

/// Remaps reviewed CloudFormation pointers and publishes both deterministic artifacts.
///
/// # Errors
/// Returns input, model, validation, publication or comparison failures.
pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let catalogs = read_catalogs(root)?;
    let cfn = report::read_text(&root.join("catalog/cloudcontrol-universal.json"))?;
    let kinds = kind_map::load_base(root, cfn.as_bytes())?;
    let checked = overlay::check(root, source).map_err(|error| Error::Review(Box::new(error)))?;
    let bytes =
        smithy::fields::publish::read_bytes(&root.join("catalog/aws-api.catalog"), 67_108_864)?
            .ok_or(Error::Invalid("catalog/aws-api.catalog is missing"))?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let endpoints = model::build(catalog, &kinds, &checked)?;
    let output = remap::build(&catalogs, &kinds, &endpoints)?;
    kinds.with_generated_relations(output.rows.as_bytes())?;
    publish(root, &output, mode)
}

fn publish(root: &Path, output: &remap::Output, mode: ReportMode) -> Result<(), Error> {
    let rows_path = root.join(ROWS_PATH);
    let report_path = root.join(REPORT_PATH);
    if mode != ReportMode::Write {
        report::publish(&rows_path, &output.rows, mode)?;
        report::publish(&report_path, &output.report, mode)?;
        return Ok(());
    }
    let mut failed_path = &rows_path;
    crate::package::atomic_write(&rows_path, |rows| -> std::io::Result<()> {
        rows.write_all(output.rows.as_bytes())?;
        rows.sync_all()?;
        failed_path = &report_path;
        crate::package::atomic_write(&report_path, |report| -> std::io::Result<()> {
            report.write_all(output.report.as_bytes())?;
            report.sync_all()
        })?;
        failed_path = &rows_path;
        Ok(())
    })
    .map_err(|source| report::ReportError::Io {
        file: failed_path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn read_catalogs(root: &Path) -> Result<Vec<LegacyCatalog>, Error> {
    ["cloudcontrol-resources", "cloudcontrol-parent-resources"]
        .into_iter()
        .map(|stem| {
            let json = root.join("catalog").join(format!("{stem}.json"));
            match smithy::fields::publish::read_bytes(&json, 4_194_304)? {
                Some(bytes) => Ok(serde_json::from_slice(&bytes)?),
                None => {
                    let path = root.join("catalog").join(format!("{stem}.toml"));
                    Ok(toml::from_str(&report::read_text(&path)?)?)
                }
            }
        })
        .collect()
}
