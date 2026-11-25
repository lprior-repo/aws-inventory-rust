use crate::store::{Store, StoreError};
use inventory_core::{Edge, Resource};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

mod identity;
use identity::{IdentityIndex, Lookup};
mod walk;
use walk::{serialized_size_exceeds, walk_properties};
mod persist;
pub use persist::persist_scan;
pub(crate) use persist::{
    RestoreOutcome, SyntheticRollback, persist_scan_owned, restore_synthetic,
};

pub const REFERENCE_RELATION: &str = "REFERENCES";

pub const REFERENCE_SOURCE: &str = "universal_reference";

pub const PROPERTIES_PREFIX: &str = "/properties";

pub const DEFAULT_MAX_DEPTH: usize = 8;

pub const DEFAULT_MAX_NODES_PER_RESOURCE: usize = 10_000;

pub const DEFAULT_MAX_PROPERTIES_BYTES: usize = 2_097_152;

pub const DEFAULT_MAX_REFERENCE_BYTES: usize = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceConfig {
    pub max_depth: usize,

    pub max_nodes_per_resource: usize,

    pub max_properties_bytes: usize,

    pub max_reference_bytes: usize,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_nodes_per_resource: DEFAULT_MAX_NODES_PER_RESOURCE,
            max_properties_bytes: DEFAULT_MAX_PROPERTIES_BYTES,
            max_reference_bytes: DEFAULT_MAX_REFERENCE_BYTES,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReferenceSummary {
    pub candidates_scanned: usize,

    pub edges_emitted: usize,

    pub references_unresolved: usize,

    pub ambiguous: usize,

    pub truncated: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReferenceResolution {
    pub summary: ReferenceSummary,

    pub edges: Vec<Edge>,
}

pub fn resolve(resources: &[Resource], config: &ReferenceConfig) -> ReferenceResolution {
    let index = IdentityIndex::build(resources);
    let mut state = ResolutionState::default();
    for resource in resources {
        state.consider(&index, resource, config);
    }
    state.finish()
}

#[derive(Default)]
struct ResolutionState {
    edges: BTreeMap<(String, String), Edge>,
    summary: ReferenceSummary,
}

impl ResolutionState {
    fn consider(&mut self, index: &IdentityIndex, resource: &Resource, config: &ReferenceConfig) {
        if serialized_size_exceeds(&resource.properties, config.max_properties_bytes) {
            self.mark_truncated();
            return;
        }
        let outcome = walk_properties(&resource.properties, config, &mut |path, value| {
            self.observe(index, resource, path, value, config);
        });
        if outcome.truncated {
            self.mark_truncated();
        }
    }

    fn mark_truncated(&mut self) {
        self.summary.truncated = self.summary.truncated.saturating_add(1);
    }

    fn observe(
        &mut self,
        index: &IdentityIndex,
        resource: &Resource,
        path: &str,
        value: &str,
        config: &ReferenceConfig,
    ) {
        self.summary.candidates_scanned = self.summary.candidates_scanned.saturating_add(1);
        let lookup = index.lookup(resource, value, config.max_reference_bytes);
        let Some((target, reference)) = self.record_lookup(lookup) else {
            return;
        };
        self.insert_edge(resource, target, reference, path);
    }

    fn record_lookup<'a>(&mut self, lookup: Lookup<'a>) -> Option<(&'a str, &'static str)> {
        match lookup {
            Lookup::Arn(target) => Some((target, "arn")),
            Lookup::Identifier(target) => Some((target, "identifier")),
            Lookup::Ambiguous => {
                self.summary.ambiguous = self.summary.ambiguous.saturating_add(1);
                None
            }
            Lookup::Unresolved => {
                self.summary.references_unresolved =
                    self.summary.references_unresolved.saturating_add(1);
                None
            }
            Lookup::Ignored => None,
        }
    }

    fn insert_edge(&mut self, resource: &Resource, target: &str, reference: &str, path: &str) {
        let key = (resource.id.clone(), target.to_owned());
        if self.edges.contains_key(&key) {
            return;
        }
        self.edges.insert(
            key,
            Edge {
                from_id: resource.id.clone(),
                to_id: target.to_owned(),
                relation: REFERENCE_RELATION.to_owned(),
                source: resource.source,
                observed_at: resource.observed_at,
                properties: json!({
                    "reference": reference,
                    "path": path,
                    "source": REFERENCE_SOURCE,
                }),
            },
        );
    }

    fn finish(mut self) -> ReferenceResolution {
        self.summary.edges_emitted = self.edges.len();
        ReferenceResolution {
            summary: self.summary,
            edges: self.edges.into_values().collect(),
        }
    }
}

pub async fn resolve_scan(
    store: &Store,
    scan_id: &str,
    config: &ReferenceConfig,
) -> Result<ReferenceResolution, StoreError> {
    let mut index = IdentityIndex::default();
    store
        .visit_resources(scan_id, |resource| {
            index.insert(&resource);
            Ok(())
        })
        .await?;
    let mut state = ResolutionState::default();
    store
        .visit_resources(scan_id, |resource| {
            state.consider(&index, &resource, config);
            Ok(())
        })
        .await?;
    Ok(state.finish())
}
