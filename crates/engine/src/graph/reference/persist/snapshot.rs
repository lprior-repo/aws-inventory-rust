use super::COLLECTOR;
use super::write::{clear, edge_count, fingerprint, insert_edge, insert_scope_page};
use crate::store::{Store, StoreError};
use inventory_core::{CollectionScope, Edge};
use sqlx::SqliteConnection;

/// The derived-reference state of one scan as it was immediately before a
/// mutation, captured inside the same write transaction as that mutation.
pub(super) struct SyntheticSnapshot {
    scopes: Vec<(String, CollectionScope, Vec<Edge>)>,
}

/// Ownership of one derived-reference mutation. `generation` is the value
/// written by the mutation itself; the compensating restore only applies while
/// that value is still current, so a later writer's state is never erased.
pub(crate) struct SyntheticRollback {
    before: SyntheticSnapshot,
    generation: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RestoreOutcome {
    Restored,
    Superseded,
}

impl SyntheticRollback {
    pub(super) fn new(before: SyntheticSnapshot, generation: i64) -> Self {
        Self { before, generation }
    }
}

pub(super) async fn snapshot_synthetic(
    tx: &mut SqliteConnection,
    scan_id: &str,
) -> Result<SyntheticSnapshot, StoreError> {
    let keys: Vec<String> = sqlx::query_scalar(
        "SELECT scope_key FROM scope_checkpoints
         WHERE scan_id=?1 AND collector=?2 ORDER BY scope_key",
    )
    .bind(scan_id)
    .bind(COLLECTOR)
    .fetch_all(&mut *tx)
    .await?;
    let mut scopes = Vec::with_capacity(keys.len());
    for key in keys {
        scopes.push(snapshot_scope(tx, scan_id, key).await?);
    }
    Ok(SyntheticSnapshot { scopes })
}

async fn snapshot_scope(
    tx: &mut SqliteConnection,
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
    .fetch_all(&mut *tx)
    .await?;
    let mut edges = Vec::with_capacity(payloads.len());
    for payload in payloads {
        edges.push(serde_json::from_str(&payload)?);
    }
    Ok((key, scope, edges))
}

/// Advances the scan's generation and returns the new value.
pub(super) async fn next_generation(
    tx: &mut SqliteConnection,
    scan_id: &str,
) -> Result<i64, StoreError> {
    Ok(sqlx::query_scalar(
        "INSERT INTO synthetic_generations(scan_id,generation) VALUES (?1,1)
         ON CONFLICT(scan_id) DO UPDATE SET generation=generation+1 RETURNING generation",
    )
    .bind(scan_id)
    .fetch_one(&mut *tx)
    .await?)
}

/// Compensates `rollback`'s mutation unless another writer changed the scan's
/// derived references afterwards. The ownership check and the restore share one
/// `BEGIN IMMEDIATE` transaction, so they are atomic across connections.
pub(crate) async fn restore_synthetic(
    store: &Store,
    scan_id: &str,
    rollback: &SyntheticRollback,
) -> Result<RestoreOutcome, StoreError> {
    let mut tx = store.writer_pool().begin_with("BEGIN IMMEDIATE").await?;
    let current: Option<i64> =
        sqlx::query_scalar("SELECT generation FROM synthetic_generations WHERE scan_id=?1")
            .bind(scan_id)
            .fetch_optional(&mut *tx)
            .await?;
    if current != Some(rollback.generation) {
        return Ok(RestoreOutcome::Superseded);
    }
    clear(scan_id, &mut tx).await?;
    for (key, scope, edges) in &rollback.before.scopes {
        insert_snapshot_scope(&mut tx, scan_id, key, scope, edges).await?;
    }
    tx.commit().await?;
    Ok(RestoreOutcome::Restored)
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
