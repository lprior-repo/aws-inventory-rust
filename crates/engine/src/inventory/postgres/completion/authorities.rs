use crate::inventory::work::{ownership_scope_bytes, persisted_scope};
use crate::inventory::{StoreError, SweepId};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};
use std::collections::BTreeMap;

const WORK: &str = "SELECT w.scope_key,w.bindings,w.authority_complete,coalesce(o.outcome,'partial') AS outcome FROM work_queue w LEFT JOIN scope_outcomes o USING(sweep_id,scope_key) WHERE w.sweep_id=$1 AND w.authority_generation=w.generation AND w.generation>0 AND ($2::bytea IS NULL OR w.scope_key>$2) ORDER BY w.scope_key LIMIT 1000";
const MISSES: &str = "UPDATE resource_scopes s SET missing_complete=CASE WHEN missing_complete<2 THEN missing_complete+1 ELSE missing_complete END,last_seen=$2,completed_sweep=$1 FROM inventory_complete_authorities a WHERE s.scope_key=a.source AND a.complete AND a.prefix AND s.observed_sweep<>$1 AND s.completed_sweep<>$1";
const MEMBER_MISSES: &str = "UPDATE inventory.scope_members sm SET missing_complete=CASE WHEN missing_complete<2 THEN missing_complete+1 ELSE missing_complete END,completed_sweep=$1 FROM inventory_complete_authorities a WHERE sm.scope_key=a.source AND a.complete AND a.prefix AND sm.observed_sweep<>$1 AND sm.completed_sweep<>$1 AND sm.missing_complete<2";

pub(super) async fn stage(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
) -> Result<(), StoreError> {
    sqlx::query("CREATE TEMP TABLE inventory_complete_authorities(source bytea PRIMARY KEY,complete boolean NOT NULL,prefix boolean NOT NULL) ON COMMIT DROP")
        .execute(&mut **tx).await?;
    let mut after: Option<Vec<u8>> = None;
    loop {
        let rows = rows(tx, sweep, after.as_deref()).await?;
        if rows.is_empty() {
            break;
        }
        insert(tx, &batch(&rows)?).await?;
        after = rows
            .last()
            .map(|row| row.try_get("scope_key"))
            .transpose()?;
    }
    Ok(())
}

async fn rows(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    after: Option<&[u8]>,
) -> Result<Vec<PgRow>, StoreError> {
    Ok(sqlx::query(WORK)
        .bind(sweep.as_str())
        .bind(after)
        .fetch_all(&mut **tx)
        .await?)
}

fn batch(rows: &[PgRow]) -> Result<BTreeMap<Vec<u8>, (bool, bool)>, StoreError> {
    let mut authorities = BTreeMap::new();
    for row in rows {
        let scope = persisted_scope(row.try_get("scope_key")?)?;
        let source = ownership_scope_bytes(&scope, row.try_get("bindings")?)?;
        let complete = row.try_get::<&str, _>("outcome")? == "complete";
        let prefix = row.try_get::<i64, _>("authority_complete")? != 0;
        reduce(&mut authorities, source, complete, prefix);
    }
    Ok(authorities)
}

fn reduce(
    authorities: &mut BTreeMap<Vec<u8>, (bool, bool)>,
    source: Vec<u8>,
    complete: bool,
    prefix: bool,
) {
    authorities
        .entry(source)
        .and_modify(|all| {
            all.0 &= complete;
            all.1 &= prefix;
        })
        .or_insert((complete, prefix));
}

async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    authorities: &BTreeMap<Vec<u8>, (bool, bool)>,
) -> Result<(), StoreError> {
    let sources: Vec<_> = authorities.keys().map(Vec::as_slice).collect();
    let complete: Vec<_> = authorities
        .values()
        .map(|(complete, _)| *complete)
        .collect();
    let prefix: Vec<_> = authorities.values().map(|(_, prefix)| *prefix).collect();
    sqlx::query("INSERT INTO inventory_complete_authorities SELECT * FROM UNNEST($1::bytea[],$2::boolean[],$3::boolean[]) ON CONFLICT(source) DO UPDATE SET complete=inventory_complete_authorities.complete AND EXCLUDED.complete,prefix=inventory_complete_authorities.prefix AND EXCLUDED.prefix")
        .bind(sources).bind(complete).bind(prefix).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn misses(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    at: i64,
) -> Result<(), StoreError> {
    sqlx::query(MISSES)
        .bind(sweep.as_str())
        .bind(at)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn member_misses(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
) -> Result<(), StoreError> {
    sqlx::query(MEMBER_MISSES)
        .bind(sweep.as_str())
        .execute(&mut **tx)
        .await?;
    Ok(())
}
