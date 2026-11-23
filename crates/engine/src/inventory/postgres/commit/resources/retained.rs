use super::{State, append, ordinal};
use crate::inventory::StoreError;
use crate::inventory::postgres::commit::keys::Keys;
use futures_util::TryStreamExt;
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;

const LOAD: &str = "SELECT u.ordinal,r.content_hash,r.first_seen,r.last_changed,r.name AS resource_name,f.* FROM UNNEST($1::text[],$2::text[],$3::text[],$4::text[]) WITH ORDINALITY u(account,region,kind,id,ordinal) JOIN resources r USING(account,region,kind,id) LEFT JOIN resource_facets f USING(account,region,kind,id) ORDER BY u.ordinal";

pub(in crate::inventory::postgres) async fn load(
    tx: &mut Transaction<'_, Postgres>,
    states: &mut BTreeMap<&ResourceKey, State<'_>>,
) -> Result<(), StoreError> {
    let ordered: Vec<_> = states.keys().copied().collect();
    let keys = Keys::new(ordered.iter().copied());
    let mut rows = sqlx::query(LOAD)
        .bind(keys.accounts)
        .bind(keys.regions)
        .bind(keys.kinds)
        .bind(keys.ids)
        .fetch(&mut **tx);
    while let Some(row) = rows.try_next().await? {
        let key = ordered
            .get(ordinal(&row)?)
            .ok_or(StoreError::Invalid("resource index exceeds bounds"))?;
        let state = states
            .get_mut(key)
            .ok_or(StoreError::Invalid("resource key is absent"))?;
        append(state, &row, None)?;
    }
    Ok(())
}
