use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::expected_runtime_catalog;

use super::files::{read_json_audit, read_toml_audit};

pub struct Catalogs {
    pub catalog: Value,

    pub catalog_sha256: String,

    pub parents: Value,

    pub parent_sha256: String,

    pub merged: Value,
}

pub fn catalogs(root: &Path) -> Result<Catalogs, HarnessError> {
    let (catalog, catalog_sha256) =
        read_json_audit(&root.join("catalog/cloudcontrol-resources.json"))?;
    let (parents, parent_sha256) =
        read_toml_audit(&root.join("catalog/cloudcontrol-parent-resources.toml"))?;
    let bound = parents
        .pointer("/evidence/root_catalog_sha256")
        .and_then(Value::as_str);
    if bound != Some(catalog_sha256.as_str()) {
        return Err(HarnessError::invalid(
            "parent catalog is not bound to the current frozen root catalog",
        ));
    }
    let merged = expected_runtime_catalog(&catalog, &parents)?;
    Ok(Catalogs {
        catalog,
        catalog_sha256,
        parents,
        parent_sha256,
        merged,
    })
}
