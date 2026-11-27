use crate::inventory::{StoreError, SweepId};
use sqlx::{Postgres, Transaction};

const MARK: &str = "UPDATE sweeps SET completed=1,ended_at=coalesce((SELECT max(o.observed_at) FROM scope_outcomes o JOIN work_queue w ON w.sweep_id=o.sweep_id AND w.scope_key=o.scope_key WHERE o.sweep_id=$1),$2::bigint) WHERE sweep_id=$1 AND completed=0 AND NOT EXISTS(SELECT 1 FROM work_queue work LEFT JOIN scope_outcomes outcomes ON outcomes.sweep_id=work.sweep_id AND outcomes.scope_key=work.scope_key WHERE work.sweep_id=$1 AND work.authority_generation=work.generation AND work.generation>0 AND coalesce(outcomes.outcome,'partial')<>'complete')";

pub(super) async fn record(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    at: i64,
) -> Result<(), StoreError> {
    sqlx::query(MARK)
        .bind(sweep.as_str())
        .bind(at)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
