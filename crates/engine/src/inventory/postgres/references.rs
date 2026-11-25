mod authorization;
mod matching;
mod selection;
mod write;

use super::PgStore;
use crate::inventory::StoreError;
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Transaction};

pub(super) async fn reconcile(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    changed: &[&ResourceKey],
) -> Result<u64, StoreError> {
    if changed.is_empty() || store.registry.permissions.catalog().is_none() {
        return Ok(0);
    }
    selection::begin(tx, changed).await?;
    let mut after = None;
    loop {
        let mut rows = selection::next(tx, after.as_ref()).await?;
        if rows.is_empty() {
            break;
        }
        authorization::load(tx, &store.registry, &mut rows).await?;
        let resolutions = matching::resolve(tx, &store.registry, &rows).await?;
        write::batch(tx, &rows, &resolutions).await?;
        after = rows.pop();
    }
    write::finish(tx).await
}
