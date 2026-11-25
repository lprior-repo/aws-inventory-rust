use super::{ReferenceConfig, ReferenceSummary, resolve_scan};
use crate::store::{Store, StoreError};
use inventory_core::{CollectionScope, Edge, Source};
use sqlx::SqliteConnection;
use std::collections::{BTreeMap, BTreeSet};

mod snapshot;
pub(crate) use snapshot::{RestoreOutcome, SyntheticRollback, restore_synthetic};
use snapshot::{next_generation, snapshot_synthetic};
#[cfg(test)]
mod tests;
mod write;
use write::{clear, edge_count, fingerprint, insert_edge, insert_scope_page};

const COLLECTOR: &str = "universal_reference";

pub async fn persist_scan(
    store: &Store,
    scan_id: &str,
    config: &ReferenceConfig,
) -> Result<ReferenceSummary, StoreError> {
    persist_scan_owned(store, scan_id, config)
        .await
        .map(|persisted| persisted.summary)
}

pub(crate) struct OwnedPersist {
    pub(crate) summary: ReferenceSummary,
    pub(crate) rollback: SyntheticRollback,
}

/// Replaces the scan's derived references and returns the means to undo exactly
/// this replacement. The previous state is captured and the generation advanced
/// inside the same write transaction as the replacement, so concurrent writers
/// on other connections serialize around it and `restore_synthetic` can tell
/// whether the state it would overwrite is still the one written here.
pub(crate) async fn persist_scan_owned(
    store: &Store,
    scan_id: &str,
    config: &ReferenceConfig,
) -> Result<OwnedPersist, StoreError> {
    let resolution = resolve_scan(store, scan_id, config).await?;
    let provenance = edge_provenance(store, scan_id, &resolution.edges).await?;
    let groups = group_edges(&resolution.edges, &provenance);
    let mut tx = store.writer_pool().begin_with("BEGIN IMMEDIATE").await?;
    let before = snapshot_synthetic(&mut tx, scan_id).await?;
    clear(scan_id, &mut tx).await?;
    write_groups(&mut tx, scan_id, groups).await?;
    let generation = next_generation(&mut tx, scan_id).await?;
    tx.commit().await?;
    Ok(OwnedPersist {
        summary: resolution.summary,
        rollback: SyntheticRollback::new(before, generation),
    })
}

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
