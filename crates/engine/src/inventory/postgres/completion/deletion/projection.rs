use crate::inventory::StoreError;
use crate::inventory::postgres::commit::keys;
use crate::inventory::postgres::{PgStore, outbox, references};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeMap;

const STAGE: &str = concat!(
    "CREATE TEMP TABLE inventory_retire_projection ON COMMIT DROP AS SELECT *,true AS deleted FROM inventory_retire_keys UNION ",
    "SELECT DISTINCT account,region,kind,id,false FROM inventory_retire_sources UNION ",
    "SELECT e.from_account,e.from_region,e.from_kind,e.from_id,false FROM edges e JOIN inventory_retire_keys d ON (d.account,d.region,d.kind,d.id)=(e.to_account,e.to_region,e.to_kind,e.to_id) WHERE NOT EXISTS(SELECT 1 FROM inventory_retire_keys f WHERE (f.account,f.region,f.kind,f.id)=(e.from_account,e.from_region,e.from_kind,e.from_id)) UNION ",
    "SELECT e.to_account,e.to_region,e.to_kind,e.to_id,false FROM edges e JOIN inventory_retire_keys d ON (d.account,d.region,d.kind,d.id)=(e.from_account,e.from_region,e.from_kind,e.from_id) WHERE NOT EXISTS(SELECT 1 FROM inventory_retire_keys t WHERE (t.account,t.region,t.kind,t.id)=(e.to_account,e.to_region,e.to_kind,e.to_id))"
);
const SELECT: &str = "SELECT * FROM inventory_retire_projection WHERE $1::text IS NULL OR (account,region,kind,id)>($1,$2,$3,$4) ORDER BY account,region,kind,id LIMIT 1000";

pub(super) async fn stage(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    sqlx::query(STAGE).execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE inventory_retire_projection ADD PRIMARY KEY(account,region,kind,id)")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(in super::super) async fn project(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<u64, StoreError> {
    let mut after = None;
    let mut inserted = 0_u64;
    loop {
        let rows = batch(tx, after.as_ref()).await?;
        if rows.is_empty() {
            break;
        }
        let keys: Vec<_> = rows.iter().map(|(key, _)| key).collect();
        inserted = inserted
            .checked_add(references::reconcile(store, tx, &keys).await?)
            .ok_or(StoreError::Invalid("reference edge count exceeds bounds"))?;
        let entries = rows
            .iter()
            .map(|(key, deleted)| (key, *deleted))
            .collect::<BTreeMap<_, _>>();
        outbox::enqueue(tx, &entries).await?;
        after = rows.into_iter().last().map(|(key, _)| key);
    }
    Ok(inserted)
}

async fn batch(
    tx: &mut Transaction<'_, Postgres>,
    after: Option<&ResourceKey>,
) -> Result<Vec<(ResourceKey, bool)>, StoreError> {
    let rows = sqlx::query(SELECT)
        .bind(after.map(|key| key.account.as_str()))
        .bind(after.map(|key| key.region.as_str()))
        .bind(after.map(|key| key.kind.as_str()))
        .bind(after.map(|key| key.id.as_str()))
        .fetch_all(&mut **tx)
        .await?;
    rows.iter()
        .map(|row| Ok((keys::decode(row)?, row.try_get("deleted")?)))
        .collect()
}
