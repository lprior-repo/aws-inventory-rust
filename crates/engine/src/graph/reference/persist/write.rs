use super::COLLECTOR;
use crate::store::{MAX_RECORD_BYTES, StoreError};
use inventory_core::{CollectionScope, Edge};
use sha2::{Digest, Sha256};
use sqlx::SqliteConnection;

const INSERT_EDGE_SQL: &str =
    "INSERT INTO edge_observations(scan_id,scope_key,page_number,from_id,to_id,relation,
         source,source_priority,observed_at,edge_json)
         VALUES (?1,?2,1,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT DO NOTHING";

pub(super) fn edge_count(len: usize) -> Result<i64, StoreError> {
    i64::try_from(len).map_err(|_| StoreError::Limit("reference edge count"))
}

pub(super) async fn clear(scan_id: &str, tx: &mut SqliteConnection) -> Result<(), StoreError> {
    for statement in [
        "DELETE FROM edge_observations WHERE scan_id=?1 AND scope_key IN
             (SELECT scope_key FROM scope_checkpoints WHERE scan_id=?1 AND collector=?2)",
        "DELETE FROM collection_pages WHERE scan_id=?1 AND scope_key IN
             (SELECT scope_key FROM scope_checkpoints WHERE scan_id=?1 AND collector=?2)",
        "DELETE FROM scope_checkpoints WHERE scan_id=?1 AND collector=?2",
    ] {
        sqlx::query(statement)
            .bind(scan_id)
            .bind(COLLECTOR)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

pub(super) async fn insert_scope_page(
    tx: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    scope: &CollectionScope,
    fingerprint: &[u8],
    edge_count: i64,
) -> Result<(), StoreError> {
    insert_scope_checkpoint(tx, scan_id, key, scope).await?;
    insert_page_row(tx, scan_id, key, fingerprint, edge_count).await
}

async fn insert_scope_checkpoint(
    tx: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    scope: &CollectionScope,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO scope_checkpoints(scan_id,scope_key,account_id,region,collector,
         parent_id,source,last_page,next_token,complete,failed)
         VALUES (?1,?2,?3,?4,?5,NULL,?6,1,NULL,1,0) ON CONFLICT DO NOTHING",
    )
    .bind(scan_id)
    .bind(key)
    .bind(&scope.account_id)
    .bind(&scope.region)
    .bind(&scope.collector)
    .bind(scope.source.as_str())
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn insert_page_row(
    tx: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    fingerprint: &[u8],
    edge_count: i64,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO collection_pages(scan_id,scope_key,page_number,fingerprint,
         next_token,resource_count,edge_count) VALUES (?1,?2,1,?3,NULL,0,?4)
         ON CONFLICT DO NOTHING",
    )
    .bind(scan_id)
    .bind(key)
    .bind(fingerprint)
    .bind(edge_count)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

pub(super) async fn insert_edge(
    tx: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    edge: &Edge,
) -> Result<(), StoreError> {
    let json = edge_json(edge)?;
    sqlx::query(INSERT_EDGE_SQL)
        .bind(scan_id)
        .bind(key)
        .bind(&edge.from_id)
        .bind(&edge.to_id)
        .bind(&edge.relation)
        .bind(edge.source.as_str())
        .bind(i64::from(edge.source.priority()))
        .bind(edge.observed_at)
        .bind(json)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

fn edge_json(edge: &Edge) -> Result<String, StoreError> {
    let json = serde_json::to_string(edge)?;
    if json.len() > MAX_RECORD_BYTES {
        return Err(StoreError::Limit("reference edge bytes"));
    }
    Ok(json)
}

pub(super) fn fingerprint(scan_id: &str, key: &str) -> Vec<u8> {
    let mut digest = Sha256::new();
    digest.update(scan_id.as_bytes());
    digest.update([0_u8]);
    digest.update(key.as_bytes());
    digest.finalize().to_vec()
}
