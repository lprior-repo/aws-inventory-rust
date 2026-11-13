//! Reproduction of the reviewed, parent-scoped Cloud Control catalog.
//!
//! One dependency level only: each child binds to a reviewed string property of
//! its documented parent and is listed with that parent context as the sole
//! required model member.

mod build;
mod descriptor;
mod evidence;
mod model;
mod plan;

use std::path::Path;

use crate::HarnessError;
use crate::catalog::archive::Archive;
use crate::catalog::json::Json;

/// The reviewed parent-dependent type count.
pub const EXPECTED_TYPES: usize = 24;

/// Build the parent catalog from the committed root catalog and archive.
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
