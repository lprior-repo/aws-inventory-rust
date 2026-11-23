use super::*;
use inventory_core::inventory::AccountId;

impl PgStore {
    pub async fn create_sweep_manifest(
        &self,
        sweep: &SweepId,
        account: &str,
    ) -> Result<(), StoreError> {
        let account = AccountId::parse(account)?;
        sqlx::query("INSERT INTO sweeps(sweep_id,account,started_at) VALUES($1,$2,$3) ON CONFLICT(sweep_id) DO NOTHING")
            .bind(sweep.as_str()).bind(account.as_str()).bind(now()?)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn sweep_manifest_account(
        &self,
        sweep: &SweepId,
    ) -> Result<Option<String>, StoreError> {
        Ok(
            sqlx::query_scalar("SELECT account FROM sweeps WHERE sweep_id=$1")
                .bind(sweep.as_str())
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn discover_active_sweep(&self) -> Result<Option<SweepId>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let active = lease::active(&mut tx, None).await?;
        tx.commit().await?;
        Ok(active)
    }
}
