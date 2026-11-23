use super::*;
use futures_util::TryStreamExt;
use inventory_core::inventory::AccountId;
use sqlx::Row;

impl PgStore {
    pub async fn resume_sweep(
        &self,
        sweep: &SweepId,
        account: &AccountId,
    ) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await?;
        lease::lock(&mut tx).await?;
        let active: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM work_queue WHERE sweep_id=$1 AND state IN(0,1))",
        )
        .bind(sweep.as_str())
        .fetch_one(&mut *tx)
        .await?;
        if active {
            lease::active(&mut tx, Some(sweep)).await?;
            check_account(&mut tx, sweep, account).await?;
            check_leases(&mut tx, sweep, now()?).await?;
            sqlx::query("UPDATE work_queue SET state=0,generation=generation+1,lease_until=0 WHERE sweep_id=$1 AND state=1")
                .bind(sweep.as_str()).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

async fn check_account(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    expected: &AccountId,
) -> Result<(), StoreError> {
    let account: Option<String> =
        sqlx::query_scalar("SELECT account FROM sweeps WHERE sweep_id=$1")
            .bind(sweep.as_str())
            .fetch_optional(&mut **tx)
            .await?;
    let account = account.as_deref().map(AccountId::parse).transpose()?;
    if account.as_ref() != Some(expected) {
        return Err(StoreError::AccountMismatch(expected.clone(), account));
    }
    Ok(())
}

async fn check_leases(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    now: i64,
) -> Result<(), StoreError> {
    let mut rows = sqlx::query("SELECT state,generation,lease_until FROM work_queue WHERE sweep_id=$1 ORDER BY scope_key FOR UPDATE")
        .bind(sweep.as_str()).fetch(&mut **tx);
    let mut live = false;
    while let Some(row) = rows.try_next().await? {
        if row.try_get::<i64, _>("state")? != 1 {
            continue;
        }
        if row.try_get::<i64, _>("generation")? == i64::MAX {
            return Err(StoreError::LeaseGenerationOverflow);
        }
        live |= row.try_get::<i64, _>("lease_until")? > now;
    }
    if live {
        return Err(StoreError::LiveLease);
    }
    Ok(())
}
