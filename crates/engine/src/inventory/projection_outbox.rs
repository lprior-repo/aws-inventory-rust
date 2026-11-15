use super::work::{
    PendingProjection, ProjectionAck, SweepId, SweepSummary, decode_resource, encode_resource,
};
use super::{BinaryStore, MAX_PROJECTION_ROWS, StoreError};
use sqlx::Row;

pub(super) async fn drain_projection(
    store: &BinaryStore,
    limit: u16,
) -> Result<Vec<PendingProjection>, StoreError> {
    if limit == 0 || i64::from(limit) > MAX_PROJECTION_ROWS {
        return Err(StoreError::Invalid("projection batch exceeds bounds"));
    }
    let rows = sqlx::query("SELECT resource_key,revision,deleted,full_rebuild FROM projection_outbox ORDER BY full_rebuild DESC,revision,resource_key LIMIT ?1")
        .bind(i64::from(limit)).fetch_all(&store.writer).await?;
    rows.into_iter()
        .map(|row| {
            let full_rebuild: i64 = row.try_get("full_rebuild")?;
            let bytes: Vec<u8> = row.try_get("resource_key")?;
            let key = if full_rebuild == 1 {
                None
            } else {
                Some(decode_resource(&bytes)?)
            };
            Ok(PendingProjection {
                key,
                revision: row.try_get("revision")?,
                deleted: row.try_get("deleted")?,
                full_rebuild: full_rebuild == 1,
            })
        })
        .collect()
}

pub(super) async fn ack_projection(
    store: &BinaryStore,
    ack: ProjectionAck,
) -> Result<u64, StoreError> {
    let key = match ack.key.as_ref() {
        Some(key) => encode_resource(key)?,
        None => vec![0_u8],
    };
    let mut tx = store.writer.begin().await?;
    let result = sqlx::query("DELETE FROM projection_outbox WHERE resource_key=?1 AND revision=?2")
        .bind(key)
        .bind(ack.revision)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 1 {
        acknowledge_revision(&mut tx, ack.key.is_none()).await?;
    }
    tx.commit().await?;
    Ok(result.rows_affected())
}

async fn acknowledge_revision(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    rebuild: bool,
) -> Result<(), StoreError> {
    if rebuild {
        sqlx::query("UPDATE metadata SET value=CAST((SELECT COUNT(*) FROM projection_outbox) AS INTEGER) WHERE key='projection_pending'").execute(&mut **tx).await?;
        sqlx::query("UPDATE metadata SET value=X'00' WHERE key='projection_rebuild' AND NOT EXISTS(SELECT 1 FROM projection_outbox WHERE full_rebuild=1)").execute(&mut **tx).await?;
    } else {
        sqlx::query("UPDATE metadata SET value=MAX(0,CAST(value AS INTEGER)-1) WHERE key='projection_pending'").execute(&mut **tx).await?;
    }
    Ok(())
}

pub(super) async fn sweep_summary(
    store: &BinaryStore,
    sweep: &SweepId,
) -> Result<SweepSummary, StoreError> {
    let rows = sqlx::query(
        "SELECT outcome,COUNT(*) AS count FROM scope_outcomes WHERE sweep_id=?1 GROUP BY outcome",
    )
    .bind(sweep.as_str())
    .fetch_all(&store.reader)
    .await?;
    let mut summary = SweepSummary::default();
    for row in rows {
        let outcome: String = row.try_get("outcome")?;
        let count = u64::try_from(row.try_get::<i64, _>("count")?)
            .map_err(|_| StoreError::Invalid("negative scope count"))?;
        update_summary(&mut summary, &outcome, count)?;
    }
    summary.resources_observed = aggregate_count(&store.reader, "SELECT COUNT(*) FROM (SELECT account,region,kind,id FROM resource_scopes WHERE observed_sweep=?1 GROUP BY account,region,kind,id)", sweep).await?;
    summary.work_pending = aggregate_count(
        &store.reader,
        "SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state IN(0,1)",
        sweep,
    )
    .await?;
    Ok(summary)
}

fn update_summary(summary: &mut SweepSummary, outcome: &str, count: u64) -> Result<(), StoreError> {
    match outcome {
        "complete" => summary.complete_scopes = count,
        "partial" => summary.partial_scopes = count,
        "inaccessible" => summary.inaccessible_scopes = count,
        "unsupported" => summary.unsupported_scopes = count,
        "failed" => summary.failed_scopes = count,
        "unobserved" => summary.unobserved_scopes = count,
        _ => return Err(StoreError::Invalid("unknown stored scope outcome")),
    }
    Ok(())
}

async fn aggregate_count(
    pool: &sqlx::SqlitePool,
    query: &'static str,
    sweep: &SweepId,
) -> Result<u64, StoreError> {
    let count: i64 = sqlx::query(query)
        .bind(sweep.as_str())
        .fetch_one(pool)
        .await?
        .try_get(0)?;
    u64::try_from(count).map_err(|_| StoreError::Invalid("aggregate count is negative"))
}
