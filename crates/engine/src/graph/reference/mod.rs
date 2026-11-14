//! Universal reference resolution over persisted observations.
//!
//! The resolver derives `REFERENCES` edges purely from observed resource
//! properties and the identities of the resources observed in the same scan.
//! It has no per-resource-type rules: any string property that exactly matches
//! an observed ARN, a resource identity, or a provider primary identifier is a
//! candidate edge. Ambiguous matches are dropped rather than guessed.
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
pub(crate) use persist::{SyntheticSnapshot, restore_synthetic, snapshot_synthetic};

/// Relation emitted by the universal resolver.
pub const REFERENCE_RELATION: &str = "REFERENCES";
/// Marker stored in edge properties to identify universal resolution.
pub const REFERENCE_SOURCE: &str = "universal_reference";
/// JSON pointer prefix for properties inside a persisted resource record.
pub const PROPERTIES_PREFIX: &str = "/properties";
/// Maximum container nesting walked inside one resource's properties.
pub const DEFAULT_MAX_DEPTH: usize = 8;
/// Maximum JSON nodes visited inside one resource's properties.
pub const DEFAULT_MAX_NODES_PER_RESOURCE: usize = 10_000;
/// Maximum serialized properties bytes processed for one resource.
pub const DEFAULT_MAX_PROPERTIES_BYTES: usize = 2_097_152;
/// Maximum candidate string bytes considered as a reference.
pub const DEFAULT_MAX_REFERENCE_BYTES: usize = 4_096;

/// Bounds applied while resolving one scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceConfig {
    /// Container nesting limit inside `properties`.
    pub max_depth: usize,
    /// JSON node budget per resource.
    pub max_nodes_per_resource: usize,
    /// Serialized `properties` byte limit; larger payloads are skipped.
    pub max_properties_bytes: usize,
    /// Longest string value considered a reference candidate.
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

/// Accounting for one resolver pass.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReferenceSummary {
    /// String property values examined as potential references.
    pub candidates_scanned: usize,
    /// Unique edges emitted.
    pub edges_emitted: usize,
    /// Reference-shaped values with no observed target.
    pub references_unresolved: usize,
    /// Identifier values that matched more than one observed resource.
    pub ambiguous: usize,
    /// Resources where a bound stopped the scan or the payload was skipped.
    pub truncated: usize,
}

/// Derived edges plus their accounting.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReferenceResolution {
    /// Bounded resolution counters, including skipped/ambiguous candidates.
    pub summary: ReferenceSummary,
    /// Deterministically ordered, deduplicated derived edges; empty means none resolved.
    pub edges: Vec<Edge>,
}

/// Resolve references across an in-memory set of observed resources.
///
/// The result is deterministic and idempotent: replaying the same resources
/// yields the same edge set.
pub fn resolve(resources: &[Resource], config: &ReferenceConfig) -> ReferenceResolution {
    let index = IdentityIndex::build(resources);
    let mut state = ResolutionState::default();
    for resource in resources {
        state.consider(&index, resource, config);
    }
    state.finish()
}

/// Accumulates edges and accounting across one bounded resource pass.
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

    /// Account for one candidate string and record its edge when it resolves.
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

    /// Accounting for one lookup; `Some` iff it names exactly one target.
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

    /// Insert one deduplicated edge for a resolved reference candidate.
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

/// Read one scan's deduplicated observations and resolve their references.
///
/// Resources are streamed twice instead of collected, so the resolver is not
/// bounded by the export snapshot budget.
/// # Errors
/// Propagates invalid/missing scan, SQL, resource decoding/validation or stream-bound errors.
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
