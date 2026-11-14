//! Frozen Cloud Control catalog binding.

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::expected_runtime_catalog;

use super::files::read_json_audit;

/// Both frozen Cloud Control catalogs and their additive runtime overlay.
pub struct Catalogs {
    /// Frozen root descriptor catalog.
    pub catalog: Value,
    /// Digest of the root catalog bytes.
    pub catalog_sha256: String,
    /// Frozen parent-dependent descriptor catalog.
    pub parents: Value,
    /// Digest of the parent catalog bytes.
    pub parent_sha256: String,
    /// Runtime overlay bound into executable catalogs.
    pub merged: Value,
}

/// Read both frozen catalogs and require their frozen binding.
pub fn catalogs(root: &Path) -> Result<Catalogs, HarnessError> {
    let (catalog, catalog_sha256) =
        read_json_audit(&root.join("catalog/cloudcontrol-resources.json"))?;
    let (parents, parent_sha256) =
        read_json_audit(&root.join("catalog/cloudcontrol-parent-resources.json"))?;
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
