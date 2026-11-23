use super::PgStore;
use crate::inventory::work::transitions::{decode_scope, encode_scope};
use crate::inventory::{StoreError, SweepId, WorkLease};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};

const CLAIM: &str = "WITH candidate AS (SELECT scope_key,generation FROM work_queue WHERE sweep_id=$1 AND (state=0 OR (state=1 AND lease_until<=$2)) ORDER BY last_complete,scope_key LIMIT 1 FOR UPDATE SKIP LOCKED) UPDATE work_queue w SET state=1,generation=w.generation+CASE WHEN w.generation=9223372036854775807 THEN 0 ELSE 1 END,authority_generation=w.generation+CASE WHEN w.generation=9223372036854775807 THEN 0 ELSE 1 END,lease_until=$3,authority_complete=CASE WHEN w.token IS NULL OR w.authority_complete=1 THEN 1 ELSE 0 END FROM candidate c WHERE w.sweep_id=$1 AND w.scope_key=c.scope_key RETURNING w.scope_key,w.generation,w.token,w.bindings,w.context,w.lease_until,c.generation AS previous_generation";

impl PgStore {
    pub async fn claim(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        let until = deadline(now_ms, lease_ms)?;
        let mut tx = self.pool.begin().await?;
        active(&mut tx, Some(sweep)).await?;
        let row = sqlx::query(CLAIM)
            .bind(sweep.as_str())
            .bind(now_ms)
            .bind(until)
            .fetch_optional(&mut *tx)
            .await?;
        let lease = row.map(|row| decode(row, sweep)).transpose()?;
        tx.commit().await?;
        Ok(lease)
    }
}

fn deadline(now: i64, duration: i64) -> Result<i64, StoreError> {
    if now < 0 || duration < 1 {
        return Err(StoreError::Invalid("invalid lease interval"));
    }
    now.checked_add(duration)
        .ok_or(StoreError::Invalid("lease interval overflow"))
}

fn decode(row: PgRow, sweep: &SweepId) -> Result<WorkLease, StoreError> {
    row.try_get::<i64, _>("previous_generation")?
        .checked_add(1)
        .ok_or(StoreError::LeaseGenerationOverflow)?;
    Ok(WorkLease {
        sweep: sweep.clone(),
        scope: decode_scope(&row.try_get::<Vec<u8>, _>("scope_key")?)?,
        generation: row.try_get("generation")?,
        token: row.try_get("token")?,
        bindings: row.try_get("bindings")?,
        context: row.try_get("context")?,
        lease_until_ms: row.try_get("lease_until")?,
    })
}

pub(super) async fn lock(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    sqlx::query("SELECT pg_advisory_xact_lock(14)")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn active(
    tx: &mut Transaction<'_, Postgres>,
    expected: Option<&SweepId>,
) -> Result<Option<SweepId>, StoreError> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1) ORDER BY sweep_id LIMIT 2",
    )
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() > 1 {
        return Err(StoreError::MultipleActiveSweeps);
    }
    let active = rows.into_iter().next().map(SweepId::new).transpose()?;
    if expected
        .zip(active.as_ref())
        .is_some_and(|(want, have)| want != have)
    {
        return Err(StoreError::ActiveSweepConflict);
    }
    Ok(active)
}

pub(super) async fn validate(
    tx: &mut Transaction<'_, Postgres>,
    lease: &WorkLease,
    now: i64,
) -> Result<(), StoreError> {
    let valid: Option<bool> = sqlx::query_scalar("SELECT state IN(1,2) AND generation=$3 AND authority_generation=generation AND lease_until>$4 AND lease_until=$5 FROM work_queue WHERE sweep_id=$1 AND scope_key=$2 FOR UPDATE")
        .bind(lease.sweep().as_str())
        .bind(encode_scope(lease.scope())?)
        .bind(lease.generation())
        .bind(now)
        .bind(lease.lease_until_ms)
        .fetch_optional(&mut **tx)
        .await?;
    if valid != Some(true) {
        return Err(StoreError::StaleLease);
    }
    Ok(())
}
