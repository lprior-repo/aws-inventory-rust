use crate::inventory::{MAX_PROJECTION_ROWS, StoreError, work::resource_bytes};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;

const INSERT: &str = "WITH input AS (SELECT * FROM UNNEST($1::bytea[],$2::boolean[]) u(key,deleted)), pending AS MATERIALIZED(SELECT count(*) AS count,bool_or(full_rebuild) AS rebuilding FROM projection_outbox), additions AS (SELECT count(*) AS count FROM input i WHERE NOT EXISTS(SELECT 1 FROM projection_outbox p WHERE p.resource_key=i.key)), decision AS MATERIALIZED(SELECT COALESCE(p.rebuilding,false) OR p.count+a.count>$3 AS rebuild FROM pending p CROSS JOIN additions a), cleared AS (DELETE FROM projection_outbox WHERE NOT full_rebuild AND (SELECT rebuild FROM decision)), selected AS (SELECT decode('00','hex') AS key,false AS deleted,true AS full_rebuild WHERE (SELECT rebuild FROM decision) UNION ALL SELECT key,deleted,false FROM input WHERE NOT (SELECT rebuild FROM decision)) INSERT INTO projection_outbox(resource_key,revision,deleted,full_rebuild) SELECT key,nextval('inventory.projection_revision_seq'),deleted,full_rebuild FROM selected WHERE true ON CONFLICT(resource_key) DO UPDATE SET revision=EXCLUDED.revision,deleted=EXCLUDED.deleted,full_rebuild=EXCLUDED.full_rebuild";

pub(super) async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    entries: &BTreeMap<&ResourceKey, bool>,
) -> Result<(), StoreError> {
    if entries.is_empty() {
        return Ok(());
    }
    let keys = entries
        .keys()
        .map(|key| resource_bytes(key))
        .collect::<Result<Vec<_>, _>>()?;
    let deleted: Vec<_> = entries.values().copied().collect();
    insert(tx, keys, deleted).await
}

async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    keys: Vec<Vec<u8>>,
    deleted: Vec<bool>,
) -> Result<(), StoreError> {
    sqlx::query("SELECT pg_advisory_xact_lock(14,1)")
        .execute(&mut **tx)
        .await?;
    sqlx::query(INSERT)
        .bind(keys)
        .bind(deleted)
        .bind(MAX_PROJECTION_ROWS)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
