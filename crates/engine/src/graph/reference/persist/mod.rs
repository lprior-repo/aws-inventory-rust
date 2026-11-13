//! Idempotent persistence of universal reference edges.
//!
//! Derived edges are written under a synthetic collection scope per
//! `(source, account, region)`. The scope key is a real `CollectionScope`
//! encoding, so provenance-aware exports decode it like any other scope. Each
//! call clears the previous synthetic scopes for the scan inside the same
//! transaction, which makes replay produce exactly the current edge set.
use super::{ReferenceConfig, ReferenceSummary, resolve_scan};
use crate::store::{Store, StoreError};
use inventory_core::{CollectionScope, Edge, Source};
use sqlx::SqliteConnection;
use std::collections::{BTreeMap, BTreeSet};

mod snapshot;
pub(crate) use snapshot::{SyntheticSnapshot, restore_synthetic, snapshot_synthetic};
mod write;
use write::{clear, edge_count, fingerprint, insert_edge, insert_scope_page};

const COLLECTOR: &str = "universal_reference";

/// Resolve references for `scan_id` and persist the derived edges.
///
/// Resources are streamed for resolution and again for the provenance of edge
/// sources; the scan is never materialized, so this is not export-budget bound.
pub async fn persist_scan(
    store: &Store,
    scan_id: &str,
    config: &ReferenceConfig,
) -> Result<ReferenceSummary, StoreError> {
    let resolution = resolve_scan(store, scan_id, config).await?;
    let provenance = edge_provenance(store, scan_id, &resolution.edges).await?;
    let groups = group_edges(&resolution.edges, &provenance);
    let mut tx = store.writer_pool().begin_with("BEGIN IMMEDIATE").await?;
    clear(scan_id, &mut tx).await?;
    write_groups(&mut tx, scan_id, groups).await?;
    tx.commit().await?;
    Ok(resolution.summary)
}

/// Provenance of every resource that sources an edge, keyed by resource id.
async fn edge_provenance(
    store: &Store,
    scan_id: &str,
    edges: &[Edge],
) -> Result<BTreeMap<String, (String, String, Source)>, StoreError> {
    let needed: BTreeSet<&str> = edges.iter().map(|edge| edge.from_id.as_str()).collect();
    let mut provenance: BTreeMap<String, (String, String, Source)> = BTreeMap::new();
    if needed.is_empty() {
        return Ok(provenance);
    }
    store
        .visit_resources(scan_id, |resource| {
            if needed.contains(resource.id.as_str()) {
                provenance.insert(
                    resource.id,
                    (resource.account_id, resource.region, resource.source),
                );
            }
            Ok(())
        })
        .await?;
    Ok(provenance)
}

/// Bucket edges by the synthetic scope their source resource belongs to.
fn group_edges<'a>(
    edges: &'a [Edge],
    provenance: &BTreeMap<String, (String, String, Source)>,
) -> BTreeMap<(Source, String, String), Vec<&'a Edge>> {
    let mut groups: BTreeMap<(Source, String, String), Vec<&Edge>> = BTreeMap::new();
    for edge in edges {
        let Some((account_id, region, source)) = provenance.get(edge.from_id.as_str()) else {
            continue;
        };
        groups
            .entry((*source, account_id.clone(), region.clone()))
            .or_default()
            .push(edge);
    }
    groups
}

async fn write_groups(
    tx: &mut SqliteConnection,
    scan_id: &str,
    groups: BTreeMap<(Source, String, String), Vec<&Edge>>,
) -> Result<(), StoreError> {
    for ((source, account_id, region), edges) in groups {
        let scope = CollectionScope {
            account_id,
            region,
            collector: COLLECTOR.to_owned(),
            parent_id: None,
            source,
        };
        let key = scope.key()?;
        let count = edge_count(edges.len())?;
        let fingerprint = fingerprint(scan_id, &key);
        insert_scope_page(tx, scan_id, &key, &scope, &fingerprint, count).await?;
        for edge in edges {
            insert_edge(tx, scan_id, &key, edge).await?;
        }
    }
    Ok(())
}
