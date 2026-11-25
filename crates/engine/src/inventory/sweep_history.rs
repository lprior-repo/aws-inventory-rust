//! Read sweep completion history from the store.

use super::{BinaryStore, StoreError};
use sqlx::Row;

/// The most recently completed sweep for an account, if any.
pub struct LastSweep {
    pub sweep_id: String,
    pub ended_at_ms: i64,
}

/// Query the store for the most recently completed sweep for an account.
/// Returns None when no completed sweep exists for that account.
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
