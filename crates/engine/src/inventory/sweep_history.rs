use super::work::SweepId;
use super::{PgStore, StoreError};
use inventory_core::inventory::AccountId;

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
pub async fn plan_run(store: &PgStore, account: &AccountId) -> Result<RunStart, StoreError> {
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

/// The account's latest successful sweep. Only drained sweeps with every scope complete
/// are marked `completed`, so failed or partial sweeps never qualify. Ties on `ended_at`
/// resolve to the greatest sweep id so the answer is deterministic.
pub async fn last_completed_sweep(
    store: &PgStore,
    account: &str,
) -> Result<Option<LastSweep>, StoreError> {
    let row: Option<(String, i64)> = sqlx::query_as(
        "SELECT sweep_id,ended_at FROM inventory.sweeps WHERE account=$1 AND completed=1 ORDER BY ended_at DESC,sweep_id DESC LIMIT 1",
    )
    .bind(account)
    .fetch_optional(&store.pool)
    .await?;
    Ok(row.map(|(sweep_id, ended_at_ms)| LastSweep {
        sweep_id,
        ended_at_ms,
    }))
}
