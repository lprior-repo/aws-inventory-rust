mod manifest;
mod resume;
mod seed;

use super::{PgStore, lease};
use crate::inventory::work::transitions::encode_scope;
use crate::inventory::{ChangeStats, StoreError, SweepId, WorkLease, WorkSpec};
use inventory_core::inventory::ScopeOutcome;
use sqlx::{Postgres, Transaction};

impl PgStore {
    pub async fn seed_works(&self, works: &[WorkSpec]) -> Result<(), StoreError> {
        let Some(first) = works.first() else {
            return Ok(());
        };
        seed::validate(self, works, first.sweep())?;
        let mut tx = self.pool.begin().await?;
        lease::lock(&mut tx).await?;
        lease::active(&mut tx, Some(first.sweep())).await?;
        for batch in works.chunks(5_000) {
            seed::insert(&mut tx, first.sweep(), batch).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn finish_scope(
        &self,
        lease: WorkLease,
        outcome: ScopeOutcome,
        observed_at_ms: i64,
    ) -> Result<ChangeStats, StoreError> {
        if observed_at_ms < 0 {
            return Err(StoreError::Invalid("negative observation timestamp"));
        }
        if outcome == ScopeOutcome::Complete {
            self.registry.permissions.validate_completion(&lease)?;
        }
        let mut tx = self.pool.begin().await?;
        lease::validate(&mut tx, &lease, observed_at_ms).await?;
        finish(&mut tx, &lease, outcome, observed_at_ms).await?;
        tx.commit().await?;
        Ok(ChangeStats::default())
    }
}

pub(super) async fn finish(
    tx: &mut Transaction<'_, Postgres>,
    lease: &WorkLease,
    outcome: ScopeOutcome,
    at: i64,
) -> Result<(), StoreError> {
    let scope = encode_scope(lease.scope())?;
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES($1,$2,$3,$4) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome=EXCLUDED.outcome,observed_at=EXCLUDED.observed_at")
        .bind(lease.sweep().as_str()).bind(&scope).bind(outcome.as_str()).bind(at)
        .execute(&mut **tx).await?;
    sqlx::query("UPDATE work_queue SET state=2,lease_until=0,last_complete=CASE WHEN $3 THEN $4 ELSE last_complete END WHERE sweep_id=$1 AND scope_key=$2")
        .bind(lease.sweep().as_str()).bind(&scope).bind(outcome == ScopeOutcome::Complete).bind(at)
        .execute(&mut **tx).await?;
    Ok(())
}

pub(super) fn now() -> Result<i64, StoreError> {
    let elapsed = std::time::UNIX_EPOCH
        .elapsed()
        .map_err(|_| StoreError::Invalid("system clock before epoch"))?;
    i64::try_from(elapsed.as_millis())
        .map_err(|_| StoreError::Invalid("timestamp exceeds i64 range"))
}
