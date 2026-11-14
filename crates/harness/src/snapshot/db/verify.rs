use std::collections::BTreeMap;

use sqlx::{Row, Sqlite, SqliteConnection, Transaction};

use super::super::CASES;
use crate::HarnessError;

pub(super) async fn verify_integrity(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), HarnessError> {
    let integrity: String = sqlx::query("PRAGMA integrity_check")
        .fetch_one(&mut **transaction)
        .await?
        .try_get(0)?;
    if integrity != "ok" {
        return Err(HarnessError::invalid(
            "snapshot database failed integrity checking",
        ));
    }
    Ok(())
}

pub(super) async fn verify_persisted(
    transaction: &mut SqliteConnection,
    scan_id: &str,
    status: &str,
) -> Result<(), HarnessError> {
    let row = sqlx::query("SELECT status FROM scans WHERE scan_id=?")
        .bind(scan_id)
        .fetch_optional(&mut *transaction)
        .await?;
    let persisted: Option<String> = match row {
        Some(row) => Some(row.try_get(0)?),
        None => None,
    };
    if persisted.as_deref() != Some(status) {
        return Err(HarnessError::invalid(
            "snapshot case disagrees with persisted scan state",
        ));
    }
    Ok(())
}

pub(super) async fn verify_persisted_scans(
    transaction: &mut Transaction<'_, Sqlite>,
    expected: BTreeMap<String, String>,
) -> Result<(), HarnessError> {
    let limit = i64::try_from(CASES.len().saturating_add(1))
        .map_err(|_| HarnessError::invalid("snapshot scan row budget exceeded"))?;
    let rows = sqlx::query("SELECT scan_id,status FROM scans LIMIT ?")
        .bind(limit)
        .fetch_all(&mut **transaction)
        .await?;
    let mut persisted = BTreeMap::new();
    for row in rows {
        persisted.insert(row.try_get::<String, _>(0)?, row.try_get::<String, _>(1)?);
    }
    if persisted != expected {
        return Err(HarnessError::invalid(
            "snapshot database must contain exactly the five asserted scan identities and states",
        ));
    }
    Ok(())
}
