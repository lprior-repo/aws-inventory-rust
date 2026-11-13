//! Refresh coverage evidence bound to one completed emulator run.

mod bind;

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;

use super::Json;
use super::files::read_json_document;
use super::rollup::{read_ordered, write_rollup};

/// Refresh coverage evidence, optionally binding one emulator summary.
pub async fn run_refresh(
    root: &Path,
    emulator_summary: Option<&Path>,
) -> Result<(Json, String), HarnessError> {
    let (matrix, catalog, catalog_sha) = load_inputs(root)?;
    let matrix = match emulator_summary {
        Some(requested) => {
            bind::bind_matrix(root, requested, &matrix, &catalog, &catalog_sha).await?
        }
        None => matrix,
    };
    write_rollup(&matrix, &root.join("docs/CLOUDCONTROL_EVIDENCE.md"))?;
    let summary = matrix
        .get("summary")
        .cloned()
        .unwrap_or(Json::Object(Vec::new()));
    Ok((matrix, super::dumps_default(&summary.to_value())))
}

fn load_inputs(root: &Path) -> Result<(Json, Value, String), HarnessError> {
    let (matrix, _) = read_ordered(&root.join("catalog/cloudcontrol-capabilities.json"))?;
    let (catalog, catalog_sha) =
        read_json_document(&root.join("catalog/cloudcontrol-resources.json"))?;
    if matrix
        .get("evidence")
        .and_then(|value| value.get("catalog_sha256"))
        .and_then(Json::as_str)
        != Some(catalog_sha.as_str())
    {
        return Err(HarnessError::invalid(
            "capability matrix does not match the current executable catalog",
        ));
    }
    Ok((matrix, catalog, catalog_sha))
}
