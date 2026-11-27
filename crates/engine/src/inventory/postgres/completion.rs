mod authorities;
mod deletion;
mod facets;
mod success;
mod unobserved;

use super::PgStore;
use crate::inventory::{ChangeStats, StoreError, SweepId};
use sqlx::{Postgres, Transaction};

pub(super) async fn drain(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    if pending(tx, sweep).await? {
        return Ok(ChangeStats::default());
    }
    unobserved::record(tx, sweep).await?;
    success::record(tx, sweep, at).await?;
    authorities::stage(tx, sweep).await?;
    authorities::misses(tx, sweep, at).await?;
    authorities::member_misses(tx, sweep).await?;
    scope_member_deletion(tx, sweep, at).await?;
    deletion::stage(tx, sweep).await?;
    let mut stats = facets::retire(store, tx, at).await?;
    stats.resources_deleted = deletion::remove(store, tx).await?;
    stats.edges_changed = deletion::project(store, tx).await?;
    sqlx::query("DROP TABLE inventory_complete_authorities,inventory_retire_keys,inventory_retire_sources,inventory_retire_projection")
        .execute(&mut **tx).await?;
    Ok(stats)
}

async fn scope_member_deletion(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    at: i64,
) -> Result<(), StoreError> {
    sqlx::query(
        "WITH vanished AS (\
         SELECT child_scope FROM inventory.scope_members WHERE missing_complete>=2 AND completed_sweep=$1 \
         EXCEPT SELECT child_scope FROM inventory.scope_members WHERE missing_complete<2\
         ) UPDATE inventory.resource_scopes rs \
         SET missing_complete=2,completed_sweep=$1,last_seen=$2 FROM vanished \
         WHERE rs.scope_key=vanished.child_scope AND rs.missing_complete<2 AND rs.observed_sweep<>$1",
    )
    .bind(sweep.as_str())
    .bind(at)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "DELETE FROM inventory.scope_members WHERE missing_complete>=2 AND completed_sweep=$1",
    )
    .bind(sweep.as_str())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn pending(tx: &mut Transaction<'_, Postgres>, sweep: &SweepId) -> Result<bool, StoreError> {
    Ok(
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM work_queue WHERE sweep_id=$1 AND state<>2)",
        )
        .bind(sweep.as_str())
        .fetch_one(&mut **tx)
        .await?,
    )
}

pub(super) fn add(stats: &mut ChangeStats, other: ChangeStats) -> Result<(), StoreError> {
    for (current, added) in [
        (&mut stats.resources_inserted, other.resources_inserted),
        (&mut stats.resources_changed, other.resources_changed),
        (&mut stats.resources_deleted, other.resources_deleted),
        (&mut stats.edges_changed, other.edges_changed),
    ] {
        *current = current
            .checked_add(added)
            .ok_or(StoreError::Invalid("completion count overflow"))?;
    }
    Ok(())
}

pub(super) async fn lock(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    sqlx::query("SELECT pg_advisory_xact_lock(key) FROM (SELECT DISTINCT hashtextextended(ROW(account,region,kind)::text,14) AS key FROM resources) groups ORDER BY key")
        .execute(&mut **tx).await?;
    Ok(())
}
