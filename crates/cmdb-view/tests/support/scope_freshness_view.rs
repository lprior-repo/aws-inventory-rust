use crate::script::{Expected, TestResult};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

const LATEST_SQL: &str = r#"
WITH latest AS (
    SELECT DISTINCT ON (scope_key) scope_key, sweep_id, outcome, observed_at
    FROM inventory.scope_outcomes
    ORDER BY scope_key, observed_at DESC, sweep_id COLLATE "C" DESC
), completed AS (
    SELECT scope_key, max(observed_at) AS last_complete_at
    FROM inventory.scope_outcomes
    WHERE outcome = 'complete'
    GROUP BY scope_key
)
SELECT latest.scope_key, latest.sweep_id, latest.outcome, latest.observed_at, completed.last_complete_at
FROM latest
LEFT JOIN completed USING (scope_key)
"#;

#[derive(Debug)]
pub struct LatestRow {
    pub account: String,
    pub region: String,
    pub service: String,
    pub operation: String,
    pub has_parent: bool,
    pub sweep_id: String,
    pub outcome: String,
    pub observed_at: i64,
    pub last_complete_at: Option<i64>,
}

impl LatestRow {
    fn order_key(&self) -> (i64, &str, &str, &str, &str) {
        (
            self.observed_at,
            &self.account,
            &self.region,
            &self.service,
            &self.operation,
        )
    }
}

struct Scope {
    account: String,
    region: String,
    service: String,
    operation: String,
    has_parent: bool,
}

pub async fn latest_rows(pool: &PgPool) -> TestResult<Vec<LatestRow>> {
    let fetched = sqlx::query(LATEST_SQL).fetch_all(pool).await?;
    let mut rows = fetched
        .iter()
        .map(latest_row)
        .collect::<TestResult<Vec<_>>>()?;
    rows.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
    Ok(rows)
}

fn latest_row(row: &PgRow) -> TestResult<LatestRow> {
    let key: Vec<u8> = row.try_get("scope_key")?;
    let scope = decode_scope(&key)?;
    Ok(LatestRow {
        account: scope.account,
        region: scope.region,
        service: scope.service,
        operation: scope.operation,
        has_parent: scope.has_parent,
        sweep_id: row.try_get("sweep_id")?,
        outcome: row.try_get("outcome")?,
        observed_at: row.try_get("observed_at")?,
        last_complete_at: row.try_get("last_complete_at")?,
    })
}

fn decode_scope(bytes: &[u8]) -> TestResult<Scope> {
    let mut rest = bytes;
    let account = take_text(&mut rest)?;
    let region = take_text(&mut rest)?;
    let service = take_text(&mut rest)?;
    let operation = take_text(&mut rest)?;
    let has_parent = match rest.first().copied().ok_or("missing parent flag")? {
        0 => false,
        1 => true,
        other => return Err(format!("invalid parent flag {other}").into()),
    };
    Ok(Scope {
        account,
        region,
        service,
        operation,
        has_parent,
    })
}

fn take_text(input: &mut &[u8]) -> TestResult<String> {
    let rest: &[u8] = *input;
    let (length, tail) = rest
        .split_at_checked(4)
        .ok_or("truncated scope text length")?;
    let length = usize::try_from(u32::from_be_bytes(length.try_into()?))?;
    let (text, tail) = tail.split_at_checked(length).ok_or("truncated scope text")?;
    *input = tail;
    Ok(std::str::from_utf8(text)?.to_owned())
}

pub fn assert_latest_row(row: &LatestRow, want: &Expected) {
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
