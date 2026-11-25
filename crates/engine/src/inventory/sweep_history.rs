use super::work::SweepId;
use super::{BinaryStore, StoreError};
use inventory_core::inventory::AccountId;
use sqlx::Row;

/// How a scheduled collection for one account must begin, given persisted state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunStart {
    /// Unfinished work already belongs to this account: continue it.
    Resume(SweepId),
    /// No unfinished work exists (fresh or fully completed database): start a new sweep.
    Fresh,
    /// Unfinished work belongs to another (or unidentified) account; starting would be unsafe.
    Blocked(SweepId),
}

/// Pure decision over the persisted active sweep and its manifest owner.
pub fn decide_start(active: Option<(SweepId, Option<&str>)>, account: &AccountId) -> RunStart {
    match active {
        None => RunStart::Fresh,
        Some((sweep, Some(owner))) if owner == account.as_str() => RunStart::Resume(sweep),
        Some((sweep, _)) => RunStart::Blocked(sweep),
    }
}

/// Resolve resume-versus-new against the database's real state.
pub async fn plan_run(store: &BinaryStore, account: &AccountId) -> Result<RunStart, StoreError> {
    let Some(sweep) = store.discover_active_sweep().await? else {
        return Ok(RunStart::Fresh);
    };
    let owner = store.sweep_manifest_account(&sweep).await?;
    Ok(decide_start(Some((sweep, owner.as_deref())), account))
}

pub struct LastSweep {
    pub sweep_id: String,
    pub ended_at_ms: i64,
}

pub async fn last_completed_sweep(
    store: &BinaryStore,
    account: &str,
) -> Result<Option<LastSweep>, StoreError> {
    let _turn = store.read_turn().await;
    let row = sqlx::query(
        "SELECT sweep_id,ended_at FROM sweeps WHERE account=?1 AND completed=1 ORDER BY ended_at DESC LIMIT 1",
    )
    .bind(account)
    .fetch_optional(&store.reader)
    .await?;

    row.map(|row| {
        let sweep_id: String = row.try_get("sweep_id").map_err(StoreError::Sqlite)?;
        let ended_at: i64 = row.try_get("ended_at").map_err(StoreError::Sqlite)?;
        Ok(LastSweep {
            sweep_id,
            ended_at_ms: ended_at,
        })
    })
    .transpose()
}
