//! Snapshot and restore of the synthetic reference scopes of one scan.
use super::COLLECTOR;
use super::write::{clear, edge_count, fingerprint, insert_edge, insert_scope_page};
use crate::store::{Store, StoreError};
use inventory_core::{CollectionScope, Edge};
use sqlx::SqliteConnection;

/// Prior synthetic scopes and edges captured before an export rewrites them.
///
/// A failed or cancelled export restores this snapshot, so publication failure
/// leaves the scan's edge set exactly as it was before the export started.
pub(crate) struct SyntheticSnapshot {
    scopes: Vec<(String, CollectionScope, Vec<Edge>)>,
}

/// Capture the universal-reference scopes and edges currently persisted.
pub(crate) async fn snapshot_synthetic(
    store: &Store,
    scan_id: &str,
) -> Result<SyntheticSnapshot, StoreError> {
    let keys = snapshot_keys(store, scan_id).await?;
    let mut scopes = Vec::with_capacity(keys.len());
    for key in keys {
        scopes.push(snapshot_scope(store, scan_id, key).await?);
    }
    Ok(SyntheticSnapshot { scopes })
}

async fn snapshot_keys(store: &Store, scan_id: &str) -> Result<Vec<String>, StoreError> {
    let keys = sqlx::query_scalar(
        "SELECT scope_key FROM scope_checkpoints
         WHERE scan_id=?1 AND collector=?2 ORDER BY scope_key",
    )
    .bind(scan_id)
    .bind(COLLECTOR)
    .fetch_all(store.export_reader())
    .await?;
    Ok(keys)
}

async fn snapshot_scope(
    store: &Store,
    scan_id: &str,
    key: String,
) -> Result<(String, CollectionScope, Vec<Edge>), StoreError> {
    let scope: CollectionScope = serde_json::from_str(&key)?;
    scope.validate()?;
    let payloads: Vec<String> = sqlx::query_scalar(
        "SELECT edge_json FROM edge_observations
         WHERE scan_id=?1 AND scope_key=?2
         ORDER BY page_number,from_id,to_id,relation",
    )
    .bind(scan_id)
    .bind(&key)
    .fetch_all(store.export_reader())
    .await?;
    let mut edges = Vec::with_capacity(payloads.len());
    for payload in payloads {
        edges.push(serde_json::from_str(&payload)?);
    }
    Ok((key, scope, edges))
}

/// Reinstate a [`SyntheticSnapshot`], replacing every current synthetic scope.
pub(crate) async fn restore_synthetic(
    store: &Store,
    scan_id: &str,
    snapshot: &SyntheticSnapshot,
) -> Result<(), StoreError> {
    let mut tx = store.writer_pool().begin_with("BEGIN IMMEDIATE").await?;
    clear(scan_id, &mut tx).await?;
    for (key, scope, edges) in &snapshot.scopes {
        insert_snapshot_scope(&mut tx, scan_id, key, scope, edges).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn insert_snapshot_scope(
    tx: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    scope: &CollectionScope,
    edges: &[Edge],
) -> Result<(), StoreError> {
    let count = edge_count(edges.len())?;
    insert_scope_page(tx, scan_id, key, scope, &fingerprint(scan_id, key), count).await?;
    for edge in edges {
        insert_edge(tx, scan_id, key, edge).await?;
    }
    Ok(())
}
