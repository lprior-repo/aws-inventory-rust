//! Scope freshness and coverage read model (M7.3).
//!
//! Reads the engine's `scope_freshness` view: one row per collection scope
//! with its latest stored outcome and the time of its last complete outcome.
//! Ages are computed here from a caller-supplied clock so the UI and tests do
//! not depend on the SQLite clock.

use super::{DatabaseSnapshot, ViewError, validate_page_limit};
use inventory_core::inventory::ScopeOutcome;
use sqlx::Row;

/// Latest stored outcome of one collection scope and how old it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeFreshness {
    pub account: String,
    pub region: String,
    pub service: String,
    pub operation: String,
    /// The scope enumerates the children of a parent resource.
    pub has_parent: bool,
    pub sweep_id: String,
    pub outcome: ScopeOutcome,
    pub observed_at_ms: i64,
    /// Observation time of the scope's last `complete` outcome, if any.
    pub last_complete_at_ms: Option<i64>,
    /// Milliseconds from the latest outcome to the report's clock (never negative).
    pub age_ms: u64,
    /// Milliseconds from the last complete outcome to the report's clock.
    pub complete_age_ms: Option<u64>,
}

/// Count of scopes whose latest stored outcome is each variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeOutcomeCounts {
    pub complete: u64,
    pub partial: u64,
    pub inaccessible: u64,
    pub unsupported: u64,
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
            ScopeOutcome::Failed => &mut self.failed,
            ScopeOutcome::Unobserved => &mut self.unobserved,
        }
    }
}

/// Stalest-first page of scope freshness with exact totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeFreshnessReport {
    /// The report's clock, as supplied by the caller.
    pub now_ms: i64,
    /// Stalest first (oldest latest outcome); at most the requested limit.
    pub scopes: Vec<ScopeFreshness>,
    /// Every scope with a stored outcome, including those not in `scopes`.
    pub total_scopes: u64,
    pub counts: ScopeOutcomeCounts,
}

pub(super) fn parse_outcome(value: &str) -> Result<ScopeOutcome, ViewError> {
    match value {
        "complete" => Ok(ScopeOutcome::Complete),
        "partial" => Ok(ScopeOutcome::Partial),
        "inaccessible" => Ok(ScopeOutcome::Inaccessible),
        "unsupported" => Ok(ScopeOutcome::Unsupported),
        "failed" => Ok(ScopeOutcome::Failed),
        "unobserved" => Ok(ScopeOutcome::Unobserved),
        _ => Err(ViewError::StoredRow),
    }
}

fn age_ms(now_ms: i64, observed_at_ms: i64) -> u64 {
    u64::try_from(now_ms.saturating_sub(observed_at_ms)).unwrap_or_default()
}

impl DatabaseSnapshot {
    /// Reads scope outcomes from the `scope_freshness` view in this snapshot.
    /// `limit` bounds the returned scopes (1..=`MAX_PAGE_LIMIT`); totals and
    /// counts always cover every scope.
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

impl super::Database {
    /// Stalest-first scope freshness; see [`DatabaseSnapshot::scope_freshness`].
    pub async fn scope_freshness(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<ScopeFreshnessReport, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let report = snapshot.scope_freshness(now_ms, limit).await?;
        snapshot.finish().await?;
        Ok(report)
    }
}
