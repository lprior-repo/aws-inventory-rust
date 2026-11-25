use super::work::{SweepId, persisted_scope};
use super::{BinaryStore, StoreError};
use inventory_core::inventory::ScopeKey;
use sqlx::{Row, Sqlite, Transaction};
use std::collections::BTreeSet;

/// A top-level scope an earlier sweep of the same account observed that the
/// given sweep did not run. Its resources are untouched by that sweep.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnobservedScope {
    pub scope: ScopeKey,
    pub last_observed_ms: i64,
    pub age_ms: i64,
}

/// Records, once a sweep has drained, every top-level scope that an earlier
/// sweep completed and this one did not run (profile removed, region narrowed).
/// `observed_at` of an `unobserved` row is the scope's last complete
/// observation, so age is derived on read and never invented here.
pub(super) async fn record(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
) -> Result<(), StoreError> {
    let current = current_scope_keys(tx, sweep).await?;
    let history = historical_last_complete(tx, sweep).await?;
    for (bytes, last) in history {
        if current.contains(&bytes) {
            continue;
        }
        let scope = persisted_scope(&bytes)?;
        if scope.parent().is_some() {
            continue;
        }
        record_unobserved(tx, sweep, &bytes, last).await?;
    }
    Ok(())
}

async fn current_scope_keys(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
) -> Result<BTreeSet<Vec<u8>>, StoreError> {
    let rows = sqlx::query("SELECT scope_key FROM work_queue WHERE sweep_id=?1")
        .bind(sweep.as_str())
        .fetch_all(&mut **tx)
        .await?;
    rows.into_iter()
        .map(|row| Ok(row.try_get::<Vec<u8>, _>("scope_key")?))
        .collect()
}

async fn historical_last_complete(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
) -> Result<Vec<(Vec<u8>, i64)>, StoreError> {
    let rows = sqlx::query("SELECT scope_key,MAX(observed_at) AS last FROM scope_outcomes WHERE sweep_id<>?1 AND outcome='complete' GROUP BY scope_key")
        .bind(sweep.as_str())
        .fetch_all(&mut **tx)
        .await?;
    rows.into_iter()
        .map(|row| {
            let bytes = row.try_get::<Vec<u8>, _>("scope_key")?;
            let last = row.try_get::<i64, _>("last")?;
            Ok((bytes, last))
        })
        .collect()
}

async fn record_unobserved(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
    scope_key: &[u8],
    last_observed: i64,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,'unobserved',?3) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='unobserved',observed_at=excluded.observed_at,detail=NULL")
        .bind(sweep.as_str())
        .bind(scope_key)
        .bind(last_observed)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn list(
    store: &BinaryStore,
    sweep: &SweepId,
    now_ms: i64,
) -> Result<Vec<UnobservedScope>, StoreError> {
    let _turn = store.read_turn().await;
    let rows = sqlx::query("SELECT scope_key,observed_at FROM scope_outcomes WHERE sweep_id=?1 AND outcome='unobserved' ORDER BY scope_key")
        .bind(sweep.as_str())
        .fetch_all(&store.reader)
        .await?;
    rows.into_iter()
        .map(|row| {
            let bytes: Vec<u8> = row.try_get("scope_key")?;
            let last_observed_ms: i64 = row.try_get("observed_at")?;
            Ok(UnobservedScope {
                scope: persisted_scope(&bytes)?,
                last_observed_ms,
                age_ms: now_ms.saturating_sub(last_observed_ms).max(0),
            })
        })
        .collect()
}
