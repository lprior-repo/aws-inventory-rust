//! Registry input validation and SQLite row decoding.
use super::*;

pub(super) fn validate_coverage(record: &CoverageRecord) -> Result<(), StoreError> {
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

pub(super) fn validate_optional_text(
    value: Option<&str>,
    field: &'static str,
) -> Result<(), StoreError> {
    if let Some(value) = value
        && (value.is_empty() || value.len() > 512 || value.contains('\0'))
    {
        return Err(StoreError::InvalidInput(field));
    }
    Ok(())
}

pub(super) fn validate_time(value: i64, field: &'static str) -> Result<(), StoreError> {
    if value < 0 {
        return Err(StoreError::InvalidInput(field));
    }
    Ok(())
}

pub(super) fn account_record(row: SqliteRow) -> Result<AccountRecord, StoreError> {
    Ok(AccountRecord {
        account_id: row.try_get(0)?,
        partition: row.try_get(1)?,
        role_arn: row.try_get(2)?,
        status: AccountStatus::parse(&row.try_get::<String, _>(3)?)?,
        created_at: row.try_get(4)?,
        updated_at: row.try_get(5)?,
    })
}

pub(super) fn coverage_record(row: SqliteRow) -> Result<CoverageRecord, StoreError> {
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
