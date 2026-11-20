//! Reads the `scope_freshness` SQL view and checks its ages against the SQLite clock.

use crate::script::{Expected, TestResult};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const VIEW_ROWS_SQL: &str = "SELECT account, region, service, operation, has_parent, sweep_id, \
     outcome, observed_at, last_complete_at, age_ms, complete_age_ms FROM scope_freshness \
     ORDER BY observed_at, account, region, service, operation";

#[derive(Debug)]
pub struct ViewRow {
    pub account: String,
    pub region: String,
    pub service: String,
    pub operation: String,
    pub has_parent: bool,
    pub sweep_id: String,
    pub outcome: String,
    pub observed_at: i64,
    pub last_complete_at: Option<i64>,
    pub age_ms: i64,
    pub complete_age_ms: Option<i64>,
}

/// Reads the view with the SQLite clock bracketed around the query.
pub async fn view_rows(path: &Path) -> TestResult<(Vec<ViewRow>, i64, i64)> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(path).read_only(true))
        .await?;
    let before = now_ms()?;
    let rows = sqlx::query(VIEW_ROWS_SQL).fetch_all(&pool).await?;
    let after = now_ms()?;
    let rows = rows
        .iter()
        .map(view_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    pool.close().await;
    Ok((rows, before, after))
}

fn view_row(row: &SqliteRow) -> Result<ViewRow, sqlx::Error> {
    Ok(ViewRow {
        account: row.try_get("account")?,
        region: row.try_get("region")?,
        service: row.try_get("service")?,
        operation: row.try_get("operation")?,
        has_parent: row.try_get::<i64, _>("has_parent")? == 1,
        sweep_id: row.try_get("sweep_id")?,
        outcome: row.try_get("outcome")?,
        observed_at: row.try_get("observed_at")?,
        last_complete_at: row.try_get("last_complete_at")?,
        age_ms: row.try_get("age_ms")?,
        complete_age_ms: row.try_get("complete_age_ms")?,
    })
}

fn now_ms() -> TestResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

pub fn assert_view_row(row: &ViewRow, want: &Expected) {
    assert_eq!(row.account, want.account);
    assert_eq!(row.region, want.region);
    assert_eq!(row.service, want.service);
    assert_eq!(row.operation, want.operation);
    assert_eq!(row.has_parent, want.has_parent);
    assert_eq!(row.sweep_id, want.sweep_id);
    assert_eq!(row.outcome, want.outcome.as_str());
    assert_eq!(row.observed_at, want.observed_at_ms);
    assert_eq!(row.last_complete_at, want.last_complete_at_ms);
}

/// The view's ages come from the SQLite clock read during the query.
pub fn assert_view_ages(row: &ViewRow, before: i64, after: i64) -> TestResult {
    let lower = before.saturating_sub(row.observed_at);
    let upper = after.saturating_sub(row.observed_at);
    assert!(
        (lower..=upper).contains(&row.age_ms),
        "age {} outside [{lower}, {upper}]",
        row.age_ms,
    );
    match (row.last_complete_at, row.complete_age_ms) {
        (None, None) => {}
        (Some(at), Some(age)) => {
            assert!((before.saturating_sub(at)..=after.saturating_sub(at)).contains(&age));
        }
        other => return Err(format!("complete age and time disagree: {other:?}").into()),
    }
    Ok(())
}
