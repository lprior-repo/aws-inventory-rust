//! Account onboarding and per-scope coverage state for the CMDB control plane.

use std::collections::BTreeMap;

use super::*;

mod records;
use records::*;

/// Explicit onboarding state for an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountStatus {
    /// Account is registered for collection.
    Onboarded,
    /// Account remains registered but has a suspended onboarding state.
    Suspended,
    /// Account is marked removed; the registry row and inventory are retained.
    Removed,
}

impl AccountStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Onboarded => "onboarded",
            Self::Suspended => "suspended",
            Self::Removed => "removed",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "onboarded" => Ok(Self::Onboarded),
            "suspended" => Ok(Self::Suspended),
            "removed" => Ok(Self::Removed),
            _ => Err(StoreError::InvalidInput("unknown account status")),
        }
    }
}

/// Coverage result for one account/region/collector scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageState {
    /// No coverage outcome is known.
    Unknown,
    /// Collection is in progress for this scope.
    Running,
    /// Scope is explicitly recorded as successfully enumerated.
    Complete,
    /// Only incomplete coverage is recorded.
    Partial,
    /// Collection was refused by authorization.
    Denied,
    /// Provider capability does not support collecting the scope.
    Unsupported,
    /// Collection failed for a reason other than the explicit denied/unsupported states.
    Failed,
}

impl CoverageState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Denied => "denied",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "unknown" => Ok(Self::Unknown),
            "running" => Ok(Self::Running),
            "complete" => Ok(Self::Complete),
            "partial" => Ok(Self::Partial),
            "denied" => Ok(Self::Denied),
            "unsupported" => Ok(Self::Unsupported),
            "failed" => Ok(Self::Failed),
            _ => Err(StoreError::InvalidInput("unknown coverage state")),
        }
    }
}

/// One registered account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRecord {
    /// Registered account identity, ordered lexically by `list_accounts`.
    pub account_id: String,
    /// Caller-supplied validated partition spelling.
    pub partition: String,
    /// Optional collection-role ARN; `None` means no role was registered.
    pub role_arn: Option<String>,
    /// Explicit onboarding state; changing it does not delete inventory.
    pub status: AccountStatus,
    /// Original nonnegative caller timestamp, retained across re-registration.
    pub created_at: i64,
    /// Last supplied update timestamp; the store does not enforce monotonicity here.
    pub updated_at: i64,
}

/// One coverage-registry row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageRecord {
    /// Registered owning account; persistence requires its account row to exist.
    pub account_id: String,
    /// Validated region identity, including caller-defined global scopes.
    pub region: String,
    /// Validated collector identity; part of the registry's composite key.
    pub collector: String,
    /// Caller-reported outcome; not derived from scan observations.
    pub state: CoverageState,
    /// Last related scan identity, or `None` if not supplied.
    pub last_scan_id: Option<String>,
    /// Last nonnegative caller attempt timestamp, or `None` if unknown.
    pub last_attempt_at: Option<i64>,
    /// Last nonnegative caller success timestamp, or `None` if unknown.
    pub last_success_at: Option<i64>,
    /// Optional failure-category text; `None` means no category was supplied.
    pub last_error_category: Option<String>,
}

/// Coverage counts by state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoverageSummary {
    /// Counts keyed by stored lowercase state; absent keys represent zero rows.
    pub by_state: BTreeMap<String, usize>,
    /// Checked total across every stored coverage row.
    pub total: usize,
}

impl Store {
    /// Upsert and onboard an account, retaining its original creation timestamp.
    /// `now` is nonnegative caller time; re-registration replaces partition/role.
    /// # Errors
    /// Rejects invalid identifiers, empty/NUL/over-512-byte roles, negative time or SQL failures.
    pub async fn register_account(
        &self,
        account_id: &str,
        partition: &str,
        role_arn: Option<&str>,
        now: i64,
    ) -> Result<(), StoreError> {
        validate_identifier(account_id, "account id")?;
        validate_identifier(partition, "partition")?;
        validate_optional_text(role_arn, "role arn")?;
        validate_time(now, "account time")?;
        sqlx::query(
            "INSERT INTO accounts(account_id,partition,role_arn,status,created_at,updated_at)
             VALUES (?1,?2,?3,'onboarded',?4,?4)
             ON CONFLICT(account_id) DO UPDATE SET partition=?2, role_arn=?3,
                 status='onboarded', updated_at=?4",
        )
        .bind(account_id)
        .bind(partition)
        .bind(role_arn)
        .bind(now)
        .execute(&self.writer)
        .await?;
        Ok(())
    }

    /// Change onboarding state without deleting the account or its inventory.
    /// # Errors
    /// Rejects invalid account IDs, negative caller time, missing accounts or SQL failures.
    pub async fn set_account_status(
        &self,
        account_id: &str,
        status: AccountStatus,
        now: i64,
    ) -> Result<(), StoreError> {
        validate_identifier(account_id, "account id")?;
        validate_time(now, "account time")?;
        let updated =
            sqlx::query("UPDATE accounts SET status=?2, updated_at=?3 WHERE account_id=?1")
                .bind(account_id)
                .bind(status.as_str())
                .bind(now)
                .execute(&self.writer)
                .await?
                .rows_affected();
        if updated == 0 {
            return Err(StoreError::MissingAccount(account_id.to_owned()));
        }
        Ok(())
    }

    /// Registered accounts in account-ID order; empty means no rows are registered.
    /// # Errors
    /// Returns SQL/decoding errors or an invalid stored onboarding-state error.
    pub async fn list_accounts(&self) -> Result<Vec<AccountRecord>, StoreError> {
        let rows = sqlx::query(
            "SELECT account_id,partition,role_arn,status,created_at,updated_at
             FROM accounts ORDER BY account_id",
        )
        .fetch_all(&self.reader)
        .await?;
        rows.into_iter().map(account_record).collect()
    }

    /// Replace coverage for the exact account/region/collector composite key.
    /// # Errors
    /// Rejects invalid IDs/optional text, negative times or SQL errors, including unknown accounts.
    pub async fn record_coverage(&self, record: &CoverageRecord) -> Result<(), StoreError> {
        validate_coverage(record)?;
        sqlx::query(
            "INSERT INTO coverage_registry(account_id,region,collector,state,last_scan_id,
                 last_attempt_at,last_success_at,last_error_category)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(account_id,region,collector) DO UPDATE SET state=?4,
                 last_scan_id=?5, last_attempt_at=?6, last_success_at=?7,
                 last_error_category=?8",
        )
        .bind(&record.account_id)
        .bind(&record.region)
        .bind(&record.collector)
        .bind(record.state.as_str())
        .bind(&record.last_scan_id)
        .bind(record.last_attempt_at)
        .bind(record.last_success_at)
        .bind(&record.last_error_category)
        .execute(&self.writer)
        .await?;
        Ok(())
    }

    /// Coverage rows for one account in region/collector order; no rows yields empty.
    /// # Errors
    /// Rejects invalid account IDs, invalid stored states or SQL/decoding failures.
    pub async fn coverage(&self, account_id: &str) -> Result<Vec<CoverageRecord>, StoreError> {
        validate_identifier(account_id, "account id")?;
        let rows = sqlx::query(
            "SELECT account_id,region,collector,state,last_scan_id,last_attempt_at,
                 last_success_at,last_error_category
             FROM coverage_registry WHERE account_id=?1 ORDER BY region,collector",
        )
        .bind(account_id)
        .fetch_all(&self.reader)
        .await?;
        rows.into_iter().map(coverage_record).collect()
    }

    /// Count all registry rows by stored state; no rows yields a zero total.
    /// # Errors
    /// Returns SQL/decoding errors or a count-conversion/total-overflow error.
    pub async fn coverage_summary(&self) -> Result<CoverageSummary, StoreError> {
        let rows = sqlx::query("SELECT state,count(*) FROM coverage_registry GROUP BY state")
            .fetch_all(&self.reader)
            .await?;
        let mut summary = CoverageSummary::default();
        for row in rows {
            let state: String = row.try_get(0)?;
            let count: i64 = row.try_get(1)?;
            let count = usize::try_from(count).map_err(|_| StoreError::Limit("coverage count"))?;
            summary.by_state.insert(state, count);
            summary.total = summary
                .total
                .checked_add(count)
                .ok_or(StoreError::Limit("coverage total"))?;
        }
        Ok(summary)
    }
}
