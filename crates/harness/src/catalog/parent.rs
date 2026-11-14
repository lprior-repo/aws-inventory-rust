mod build;
mod descriptor;
mod evidence;
mod model;
mod plan;

use std::path::Path;

use crate::HarnessError;
use crate::catalog::archive::Archive;
use crate::catalog::json::Json;

pub const EXPECTED_TYPES: usize = 24;

pub fn build(root: &Path, archive: &Archive) -> Result<Json, HarnessError> {
    build::verify_archive(root)?;
    let (root_catalog, root_sha) = build::root_catalog(root)?;
    let known = build::known_types(&root_catalog);
    let global = build::global_names();
    let (mut resources, mut plans) = build::collect(archive, &known, &global)?;
    build::check_count(&resources)?;
    resources.sort_by(|left, right| left.0.cmp(&right.0));
    plans.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(evidence::document(&root_sha, resources, plans))
}
