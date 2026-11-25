use crate::inventory::work::persisted_scope;
use crate::inventory::{StoreError, SweepId};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};

const HISTORY: &str = "SELECT o.scope_key,max(o.observed_at) AS last FROM scope_outcomes o WHERE o.sweep_id<>$1 AND o.outcome='complete' AND ($2::bytea IS NULL OR o.scope_key>$2) AND NOT EXISTS(SELECT 1 FROM work_queue w WHERE w.sweep_id=$1 AND w.scope_key=o.scope_key) GROUP BY o.scope_key ORDER BY o.scope_key LIMIT 1000";

pub(super) async fn record(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
) -> Result<(), StoreError> {
    let mut after: Option<Vec<u8>> = None;
    loop {
        let rows = sqlx::query(HISTORY)
            .bind(sweep.as_str())
            .bind(after.as_deref())
            .fetch_all(&mut **tx)
            .await?;
        if rows.is_empty() {
            break;
        }
        insert(tx, sweep, &rows).await?;
        after = rows
            .last()
            .map(|row| row.try_get("scope_key"))
            .transpose()?;
    }
    Ok(())
}

async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    rows: &[PgRow],
) -> Result<(), StoreError> {
    let mut scopes = Vec::new();
    let mut times = Vec::new();
    for row in rows {
        let bytes: &[u8] = row.try_get("scope_key")?;
        if persisted_scope(bytes)?.parent().is_none() {
            scopes.push(bytes);
            times.push(row.try_get::<i64, _>("last")?);
        }
    }
    if !scopes.is_empty() {
        sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) SELECT $1,s,'unobserved',t FROM UNNEST($2::bytea[],$3::bigint[]) u(s,t) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='unobserved',observed_at=EXCLUDED.observed_at,detail=NULL")
            .bind(sweep.as_str()).bind(scopes).bind(times).execute(&mut **tx).await?;
    }
    Ok(())
}
