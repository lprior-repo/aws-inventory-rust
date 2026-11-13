//! Account onboarding and per-scope coverage state for the CMDB control plane.

use std::collections::BTreeMap;

use super::*;

/// Explicit onboarding state for an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountStatus {
    Onboarded,
    Suspended,
    Removed,
}

impl AccountStatus {
    pub const fn as_str(self) -> &'static str {
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
    Unknown,
    Running,
    Complete,
    Partial,
    Denied,
    Unsupported,
    Failed,
}

impl CoverageState {
    pub const fn as_str(self) -> &'static str {
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

    pub const fn is_terminal_success(self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// One registered account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRecord {
    pub account_id: String,
    pub partition: String,
    pub role_arn: Option<String>,
    pub status: AccountStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

/// One coverage-registry row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageRecord {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub state: CoverageState,
    pub last_scan_id: Option<String>,
    pub last_attempt_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub last_error_category: Option<String>,
}

/// Coverage counts by state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoverageSummary {
    pub by_state: BTreeMap<String, usize>,
    pub total: usize,
}

impl Store {
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

    pub async fn list_accounts(&self) -> Result<Vec<AccountRecord>, StoreError> {
        let rows = sqlx::query(
            "SELECT account_id,partition,role_arn,status,created_at,updated_at
             FROM accounts ORDER BY account_id",
        )
        .fetch_all(&self.reader)
        .await?;
        rows.into_iter().map(account_record).collect()
    }

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

fn validate_coverage(record: &CoverageRecord) -> Result<(), StoreError> {
    validate_identifier(&record.account_id, "account id")?;
    validate_identifier(&record.region, "region")?;
    validate_identifier(&record.collector, "collector")?;
    validate_optional_text(record.last_scan_id.as_deref(), "scan id")?;
    validate_optional_text(record.last_error_category.as_deref(), "error category")?;
    for value in [record.last_attempt_at, record.last_success_at]
        .into_iter()
        .flatten()
    {
        validate_time(value, "coverage time")?;
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, field: &'static str) -> Result<(), StoreError> {
    if let Some(value) = value
        && (value.is_empty() || value.len() > 512 || value.contains('\0'))
    {
        return Err(StoreError::InvalidInput(field));
    }
    Ok(())
}

fn validate_time(value: i64, field: &'static str) -> Result<(), StoreError> {
    if value < 0 {
        return Err(StoreError::InvalidInput(field));
    }
    Ok(())
}

fn account_record(row: SqliteRow) -> Result<AccountRecord, StoreError> {
    Ok(AccountRecord {
        account_id: row.try_get(0)?,
        partition: row.try_get(1)?,
        role_arn: row.try_get(2)?,
        status: AccountStatus::parse(&row.try_get::<String, _>(3)?)?,
        created_at: row.try_get(4)?,
        updated_at: row.try_get(5)?,
    })
}

fn coverage_record(row: SqliteRow) -> Result<CoverageRecord, StoreError> {
    Ok(CoverageRecord {
        account_id: row.try_get(0)?,
        region: row.try_get(1)?,
        collector: row.try_get(2)?,
        state: CoverageState::parse(&row.try_get::<String, _>(3)?)?,
        last_scan_id: row.try_get(4)?,
        last_attempt_at: row.try_get(5)?,
        last_success_at: row.try_get(6)?,
        last_error_category: row.try_get(7)?,
    })
}
