use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, anyhow, ensure};
use inventory_core::Selection;
use serde_json::{Value, json};

use crate::{catalog, cli::PlanArgs};

const CLOUD_CONTROL_CATALOG: &str = include_str!("../catalog/cloudcontrol-resources.toml");
pub(crate) const PARENT_CATALOG: &str =
    include_str!("../catalog/cloudcontrol-parent-resources.toml");

pub fn create(options: PlanArgs, selection: &Selection) -> Result<Value> {
    validate_options(&options)?;
    let mut catalog = build::catalog(&options, selection)?;
    let targets = accounting::targets(&options);
    let requests = accounting::request_accounting(&catalog, &targets);
    let universal = catalog.universal.take();
    let mut plan = assemble::plan(&options, catalog, &targets, &requests)?;
    if let Some(universal) = universal {
        universal::annotate(&mut plan, universal.list_types, universal.catalog)?;
    }
    Ok(plan)
}

#[cfg(test)]
mod tests;

mod descriptors;
pub use descriptors::full_catalog;

mod universal;

mod selection;
use selection::{array, cloud_plan, native_plan, select_groups, string, validate_options};

mod accounting;
mod assemble;
mod build;
