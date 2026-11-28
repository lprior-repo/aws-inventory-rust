use super::{DatabaseSnapshot, ViewError, validate_page_limit};
use inventory_core::inventory::ScopeOutcome;
use sqlx::Row;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeFreshness {
    pub account: String,
    pub region: String,
    pub service: String,
    pub operation: String,
    pub has_parent: bool,
    pub sweep_id: String,
    pub outcome: ScopeOutcome,
    pub observed_at_ms: i64,
    pub last_complete_at_ms: Option<i64>,
    pub age_ms: u64,
    pub complete_age_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeOutcomeCounts {
    pub complete: u64,
    pub partial: u64,
    pub inaccessible: u64,
    pub unsupported: u64,
    pub not_applicable: u64,
    pub failed: u64,
    pub unobserved: u64,
}

impl ScopeOutcomeCounts {
    fn slot(&mut self, outcome: ScopeOutcome) -> &mut u64 {
        match outcome {
            ScopeOutcome::Complete => &mut self.complete,
            ScopeOutcome::Partial => &mut self.partial,
            ScopeOutcome::Inaccessible => &mut self.inaccessible,
            ScopeOutcome::Unsupported => &mut self.unsupported,
            ScopeOutcome::NotApplicable => &mut self.not_applicable,
            ScopeOutcome::Failed => &mut self.failed,
            ScopeOutcome::Unobserved => &mut self.unobserved,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeFreshnessReport {
    pub now_ms: i64,
    pub scopes: Vec<ScopeFreshness>,
    pub total_scopes: u64,
    pub counts: ScopeOutcomeCounts,
}

pub(super) fn parse_outcome(value: &str) -> Result<ScopeOutcome, ViewError> {
    match value {
        "complete" => Ok(ScopeOutcome::Complete),
        "partial" => Ok(ScopeOutcome::Partial),
        "inaccessible" => Ok(ScopeOutcome::Inaccessible),
        "unsupported" => Ok(ScopeOutcome::Unsupported),
        "not_applicable" => Ok(ScopeOutcome::NotApplicable),
        "failed" => Ok(ScopeOutcome::Failed),
        "unobserved" => Ok(ScopeOutcome::Unobserved),
        _ => Err(ViewError::StoredRow),
    }
}

fn age_ms(now_ms: i64, observed_at_ms: i64) -> u64 {
    u64::try_from(now_ms.saturating_sub(observed_at_ms)).unwrap_or_default()
}

impl DatabaseSnapshot {
    pub async fn scope_freshness(
        &mut self,
        now_ms: i64,
        limit: usize,
    ) -> Result<ScopeFreshnessReport, ViewError> {
        validate_page_limit(limit)?;
        let limit = i64::try_from(limit).map_err(|_| ViewError::PageLimit)?;
        let mut counts = ScopeOutcomeCounts::default();
        let mut total_scopes = 0_u64;
        let grouped =
            sqlx::query("SELECT outcome, count(*) AS scopes FROM scope_freshness GROUP BY outcome")
                .fetch_all(&mut *self.tx)
                .await?;
        for row in grouped {
            let outcome = parse_outcome(row.try_get::<&str, _>("outcome")?)?;
            let scopes = u64::try_from(row.try_get::<i64, _>("scopes")?)
                .map_err(|_| ViewError::StoredRow)?;
            let slot = counts.slot(outcome);
            *slot = slot.checked_add(scopes).ok_or(ViewError::StoredRow)?;
            total_scopes = total_scopes
                .checked_add(scopes)
                .ok_or(ViewError::StoredRow)?;
        }
        let rows = sqlx::query(
            "SELECT account, region, service, operation, has_parent, sweep_id, outcome, \
             observed_at, last_complete_at FROM scope_freshness \
             ORDER BY observed_at ASC, account, region, service, operation, scope_key LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&mut *self.tx)
        .await?;
        let scopes = rows
            .into_iter()
            .map(|row| {
                let observed_at_ms: i64 = row.try_get("observed_at")?;
                let last_complete_at_ms: Option<i64> = row.try_get("last_complete_at")?;
                Ok(ScopeFreshness {
                    account: row.try_get("account")?,
                    region: row.try_get("region")?,
                    service: row.try_get("service")?,
                    operation: row.try_get("operation")?,
                    has_parent: row.try_get("has_parent")?,
                    sweep_id: row.try_get("sweep_id")?,
                    outcome: parse_outcome(row.try_get::<&str, _>("outcome")?)?,
                    observed_at_ms,
                    last_complete_at_ms,
                    age_ms: age_ms(now_ms, observed_at_ms),
                    complete_age_ms: last_complete_at_ms.map(|at| age_ms(now_ms, at)),
                })
            })
            .collect::<Result<Vec<_>, ViewError>>()?;
        Ok(ScopeFreshnessReport {
            now_ms,
            scopes,
            total_scopes,
            counts,
        })
    }
}
