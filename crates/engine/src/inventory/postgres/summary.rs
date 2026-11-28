use super::PgStore;
use crate::inventory::{
    StoreError, SweepId, SweepSummary, WorkLease, work::transitions::encode_scope,
};
use sqlx::{Row, postgres::PgRow};

const SUMMARY: &str = "SELECT count(*) FILTER(WHERE outcome='complete') AS complete_scopes,count(*) FILTER(WHERE outcome='partial') AS partial_scopes,count(*) FILTER(WHERE outcome='inaccessible') AS inaccessible_scopes,count(*) FILTER(WHERE outcome='unsupported') AS unsupported_scopes,count(*) FILTER(WHERE outcome='not_applicable') AS not_applicable_scopes,count(*) FILTER(WHERE outcome='failed') AS failed_scopes,count(*) FILTER(WHERE outcome='unobserved') AS unobserved_scopes,(SELECT count(*) FROM (SELECT account,region,kind,id FROM resource_scopes WHERE observed_sweep=$1 GROUP BY account,region,kind,id) r) AS resources_observed,(SELECT count(*) FROM work_queue WHERE sweep_id=$1 AND state IN(0,1)) AS work_pending FROM scope_outcomes WHERE sweep_id=$1";

impl PgStore {
    pub async fn sweep_summary(&self, sweep: &SweepId) -> Result<SweepSummary, StoreError> {
        let row = sqlx::query(SUMMARY)
            .bind(sweep.as_str())
            .fetch_one(&self.pool)
            .await?;
        Ok(SweepSummary {
            complete_scopes: count(&row, "complete_scopes")?,
            partial_scopes: count(&row, "partial_scopes")?,
            inaccessible_scopes: count(&row, "inaccessible_scopes")?,
            unsupported_scopes: count(&row, "unsupported_scopes")?,
            not_applicable_scopes: count(&row, "not_applicable_scopes")?,
            failed_scopes: count(&row, "failed_scopes")?,
            unobserved_scopes: count(&row, "unobserved_scopes")?,
            resources_observed: count(&row, "resources_observed")?,
            work_pending: count(&row, "work_pending")?,
        })
    }

    pub async fn release_lease(&self, lease: WorkLease) -> Result<(), StoreError> {
        let generation = lease
            .generation
            .checked_add(1)
            .ok_or(StoreError::LeaseGenerationOverflow)?;
        let result = sqlx::query("UPDATE work_queue SET state=0,generation=$3,lease_until=0 WHERE sweep_id=$1 AND scope_key=$2 AND state=1 AND generation=$4 AND lease_until=$5")
            .bind(lease.sweep.as_str()).bind(encode_scope(&lease.scope)?).bind(generation)
            .bind(lease.generation).bind(lease.lease_until_ms).execute(&self.pool).await?;
        if result.rows_affected() != 1 {
            return Err(StoreError::StaleLease);
        }
        Ok(())
    }
}

fn count(row: &PgRow, name: &str) -> Result<u64, StoreError> {
    u64::try_from(row.try_get::<i64, _>(name)?)
        .map_err(|_| StoreError::Invalid("negative sweep summary count"))
}
