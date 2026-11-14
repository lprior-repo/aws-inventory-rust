use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

pub(super) const PARENT_TYPE_COUNT: usize = 24;
const PARENT_CATALOG_JSON: &str =
    include_str!("../../../../catalog/cloudcontrol-parent-resources.json");
const CHILD_TYPE_CONCURRENCY: usize = 2;
const MAX_MODELS_PER_ROOT: usize = 10_000;
const MAX_MODEL_BYTES: usize = 8_192;
const MAX_BINDINGS: usize = 4;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Binding {
    source_pointer: String,
    model_property: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParentPlan {
    pub type_name: String,
    pub parent_type: String,
    bindings: Vec<Binding>,
    relation: String,
    evidence: String,
}

#[derive(Deserialize)]
struct ParentCatalog {
    schema_version: u32,
    evidence: CatalogEvidence,
    resource_types: Vec<ResourceType>,
    parent_plans: Vec<ParentPlan>,
}

pub(super) struct ChildType<'a> {
    descriptor: &'a ResourceType,
    plan: ParentPlan,
}

pub(super) struct RootGroup<'a> {
    descriptor: &'a ResourceType,
    children: Vec<ChildType<'a>>,
}

#[derive(Clone, Debug)]
pub(super) struct ListRequest {
    pub type_name: String,
    pub resource_model: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct GetRequest {
    pub type_name: String,
    pub identifier: String,
}

#[derive(Clone, Debug)]
struct ParentContext {
    resource_id: String,
    resource_kind: String,
    model: BTreeMap<String, String>,
    encoded_model: String,
    scope_parent: String,
}

#[derive(Default)]
struct GroupLedger {
    models: BTreeSet<[u8; 32]>,
    items: BTreeSet<[u8; 32]>,
}

#[derive(Default)]
struct DiscoveryState {
    exhausted: AtomicBool,
    missing: AtomicBool,
    ledger: Mutex<GroupLedger>,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod wire_tests;

mod catalog;
pub(super) use catalog::{catalog, plans, validate_catalog};
mod grouping;
pub(super) use grouping::groups;
use grouping::{admit_children, admit_model, parent_context};
mod discovery;
use discovery::check_cancelled;
pub(super) use discovery::collect_group_with;
mod children;
use children::{collect_child, emit_failed_read};

#[cfg(test)]
use children::attach_parent;

#[cfg(test)]
use catalog::{find_type, validate_bindings};
