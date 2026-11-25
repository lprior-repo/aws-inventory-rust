use crate::inventory::postgres::PgStore;
use crate::inventory::postgres::commit::{keys, kind_rows, references, resource_write, resources};
use crate::inventory::{ChangeStats, StoreError};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;

const SELECT: &str = "SELECT DISTINCT account,region,kind,id FROM inventory_retire_sources WHERE $1::text IS NULL OR (account,region,kind,id)>($1,$2,$3,$4) ORDER BY account,region,kind,id LIMIT 1000";

pub(super) async fn retire(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    sqlx::query("DELETE FROM resource_facets f USING inventory_retire_sources r WHERE (f.account,f.region,f.kind,f.id,f.source_key)=(r.account,r.region,r.kind,r.id,r.scope_key)")
        .execute(&mut **tx).await?;
    let mut after = None;
    let mut stats = ChangeStats::default();
    loop {
        let keys = batch(tx, after.as_ref()).await?;
        if keys.is_empty() {
            break;
        }
        let changed = apply(store, tx, &keys, at).await?;
        stats.resources_changed = stats
            .resources_changed
            .checked_add(changed)
            .ok_or(StoreError::Invalid("resource count exceeds bounds"))?;
        after = keys.into_iter().last();
    }
    Ok(stats)
}

async fn batch(
    tx: &mut Transaction<'_, Postgres>,
    after: Option<&ResourceKey>,
) -> Result<Vec<ResourceKey>, StoreError> {
    let rows = sqlx::query(SELECT)
        .bind(after.map(|key| key.account.as_str()))
        .bind(after.map(|key| key.region.as_str()))
        .bind(after.map(|key| key.kind.as_str()))
        .bind(after.map(|key| key.id.as_str()))
        .fetch_all(&mut **tx)
        .await?;
    rows.iter().map(keys::decode).collect()
}

async fn apply(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    keys: &[ResourceKey],
    at: i64,
) -> Result<u64, StoreError> {
    let mut states: BTreeMap<_, _> = keys
        .iter()
        .map(|key| (key, resources::initial(key, at)))
        .collect();
    resources::retained(tx, &mut states).await?;
    let changed = prepare(&store.registry, &mut states, at)?;
    resource_write::retire(tx, &states).await?;
    // Retirement runs inside the page transaction: its staging rows would
    // survive from the earlier write until commit. Keep retirement on UNNEST.
    kind_rows::persist(tx, &store.registry, &states, None).await?;
    references::persist(tx, &store.registry, &states).await?;
    Ok(changed)
}

fn prepare(
    registry: &crate::inventory::SchemaRegistry,
    states: &mut BTreeMap<&ResourceKey, resources::State<'_>>,
    at: i64,
) -> Result<u64, StoreError> {
    let mut changed = 0_u64;
    for state in states.values_mut() {
        state.facet_changed = true;
        resources::prepare(registry, state, at)?;
        if state
            .prepared
            .as_ref()
            .is_some_and(|row| row.content_changed)
        {
            changed = changed
                .checked_add(1)
                .ok_or(StoreError::Invalid("resource count exceeds bounds"))?;
        }
    }
    Ok(changed)
}
