use super::*;
use sqlx::sqlite::SqliteRow;

const READY_STALEST_FIRST: &str = "SELECT w.scope_key,w.token,w.bindings,w.generation,w.authority_complete,w.context,w.last_complete FROM work_queue w WHERE w.sweep_id=?1 AND w.state=0 ORDER BY w.last_complete ASC,w.scope_key LIMIT 1";
const EXPIRED_STALEST_FIRST: &str = "SELECT w.scope_key,w.token,w.bindings,w.generation,w.authority_complete,w.context,w.last_complete FROM work_queue w WHERE w.sweep_id=?1 AND w.state=1 AND w.lease_until<=?2 ORDER BY w.last_complete ASC,w.scope_key LIMIT 1";
const CLAIM: &str = "UPDATE work_queue SET state=1,generation=?3,authority_generation=?3,lease_until=?4,authority_complete=?5 WHERE sweep_id=?1 AND scope_key=?2";
const CLAIM_WITH_DEPTH: &str = "UPDATE work_queue SET state=1,generation=?3,authority_generation=?3,lease_until=?4,authority_complete=?5 WHERE sweep_id=?1 AND scope_key=?2 RETURNING (SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state IN(0,1)) AS pending_work";

impl BinaryStore {
    pub async fn claim_with_depth(
        &self,
        sweep: &SweepId,
        now_ms: i64,
        lease_ms: i64,
    ) -> Result<Option<WorkLease>, StoreError> {
        claim_inner(self, sweep, now_ms, lease_ms, true).await
    }
}

pub(in crate::inventory) async fn claim(
    store: &BinaryStore,
    sweep: &SweepId,
    now_ms: i64,
    lease_ms: i64,
) -> Result<Option<WorkLease>, StoreError> {
    claim_inner(store, sweep, now_ms, lease_ms, false).await
}

async fn claim_inner(
    store: &BinaryStore,
    sweep: &SweepId,
    now_ms: i64,
    lease_ms: i64,
    depth: bool,
) -> Result<Option<WorkLease>, StoreError> {
    let lease_until = now_ms
        .checked_add(lease_ms)
        .filter(|_| now_ms >= 0 && lease_ms > 0)
        .ok_or(StoreError::Invalid("invalid lease interval"))?;
    let (_turn, mut tx) = store.begin_write().await?;
    validate_active_sweep(&mut tx, Some(sweep)).await?;
    let lease = match select(&mut tx, sweep, now_ms).await? {
        Some(row) => Some(lease(&mut tx, sweep, &row, lease_until, depth).await?),
        None => None,
    };
    tx.commit().await?;
    Ok(lease)
}

async fn select(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
    now_ms: i64,
) -> Result<Option<SqliteRow>, StoreError> {
    let ready = sqlx::query(READY_STALEST_FIRST)
        .bind(sweep.as_str())
        .fetch_optional(&mut **tx)
        .await?;
    let expired = sqlx::query(EXPIRED_STALEST_FIRST)
        .bind(sweep.as_str())
        .bind(now_ms)
        .fetch_optional(&mut **tx)
        .await?;
    choose(ready, expired)
}

fn choose(
    ready: Option<SqliteRow>,
    expired: Option<SqliteRow>,
) -> Result<Option<SqliteRow>, StoreError> {
    match (ready, expired) {
        (Some(ready), Some(expired)) => {
            if staleness_key(&ready)? < staleness_key(&expired)? {
                Ok(Some(ready))
            } else {
                Ok(Some(expired))
            }
        }
        (ready, expired) => Ok(ready.or(expired)),
    }
}

fn staleness_key(row: &SqliteRow) -> Result<(i64, Vec<u8>), StoreError> {
    Ok((row.try_get("last_complete")?, row.try_get("scope_key")?))
}

async fn lease(
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
    row: &SqliteRow,
    lease_until: i64,
    depth: bool,
) -> Result<WorkLease, StoreError> {
    let key: Vec<u8> = row.try_get("scope_key")?;
    let complete = row.try_get::<Option<&[u8]>, _>("token")?.is_none()
        || row.try_get::<bool, _>("authority_complete")?;
    let mut lease = prepare(sweep, row, &key, lease_until)?;
    lease.pending_work = update(tx, &key, &lease, complete, depth).await?;
    Ok(lease)
}

fn prepare(
    sweep: &SweepId,
    row: &SqliteRow,
    key: &[u8],
    lease_until_ms: i64,
) -> Result<WorkLease, StoreError> {
    let generation = row
        .try_get::<i64, _>("generation")?
        .checked_add(1)
        .ok_or(StoreError::Invalid("lease generation overflow"))?;
    Ok(WorkLease {
        sweep: sweep.clone(),
        scope: decode_scope(key)?,
        generation,
        token: row.try_get("token")?,
        bindings: row.try_get("bindings")?,
        context: row.try_get("context")?,
        lease_until_ms,
        pending_work: None,
    })
}

async fn update(
    tx: &mut Transaction<'_, Sqlite>,
    key: &[u8],
    lease: &WorkLease,
    complete: bool,
    depth: bool,
) -> Result<Option<u64>, StoreError> {
    if depth {
        return claim_depth(tx, key, lease, complete).await.map(Some);
    }
    sqlx::query(CLAIM)
        .bind(lease.sweep.as_str())
        .bind(key)
        .bind(lease.generation)
        .bind(lease.lease_until_ms)
        .bind(complete)
        .execute(&mut **tx)
        .await?;
    Ok(None)
}

async fn claim_depth(
    tx: &mut Transaction<'_, Sqlite>,
    key: &[u8],
    lease: &WorkLease,
    complete: bool,
) -> Result<u64, StoreError> {
    let row = sqlx::query(CLAIM_WITH_DEPTH)
        .bind(lease.sweep.as_str())
        .bind(key)
        .bind(lease.generation)
        .bind(lease.lease_until_ms)
        .bind(complete)
        .fetch_one(&mut **tx)
        .await?;
    u64::try_from(row.try_get::<i64, _>("pending_work")?)
        .map_err(|_| StoreError::Invalid("negative unfinished-work count"))
}
