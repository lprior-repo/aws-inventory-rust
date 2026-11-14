use super::*;

impl JobStore {
    #[tracing::instrument(skip(self), fields(job_id))]
    pub async fn get(&self, job_id: &str) -> Result<JobRecord, JobError> {
        validate_key(job_id, "job ID")?;
        let row = sqlx::query("SELECT * FROM inventory_jobs WHERE job_id=?1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| JobError::Missing(job_id.into()))?;
        decode_job(&row)
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, limit: usize) -> Result<Vec<JobRecord>, JobError> {
        if !(1..=MAX_LIST_ROWS).contains(&limit) {
            return Err(JobError::Invalid("list limit must be 1..1000"));
        }
        let limit = i64::try_from(limit).map_err(|_| JobError::Limit("list rows"))?;
        let rows = sqlx::query("SELECT job_id,kind,state,attempt,max_attempts,available_at,created_at,updated_at,lease_owner,lease_until,cancel_requested,last_error,NULL AS payload,NULL AS result FROM inventory_jobs ORDER BY created_at,job_id LIMIT ?1")
            .bind(limit).fetch_all(&self.pool).await?;
        rows.iter().map(decode_job).collect()
    }

    #[tracing::instrument(skip(self), fields(job_id))]
    pub async fn history(&self, job_id: &str) -> Result<Vec<AttemptRecord>, JobError> {
        validate_key(job_id, "job ID")?;
        let rows = sqlx::query("SELECT attempt,worker,started_at,finished_at,state,error FROM inventory_job_attempts WHERE job_id=?1 ORDER BY attempt LIMIT 32")
            .bind(job_id).fetch_all(&self.pool).await?;
        rows.iter()
            .map(|row| {
                Ok(AttemptRecord {
                    attempt: row.try_get("attempt")?,
                    worker: row.try_get("worker")?,
                    started_at: row.try_get("started_at")?,
                    finished_at: row.try_get("finished_at")?,
                    state: row.try_get("state")?,
                    error: row.try_get("error")?,
                })
            })
            .collect()
    }
}
#[tracing::instrument(skip_all)]
pub(super) async fn optional_row(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Option<SqliteRow>, JobError> {
    Ok(sqlx::query("SELECT * FROM inventory_jobs WHERE job_id=?1")
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?)
}

#[tracing::instrument(skip_all)]
pub(super) async fn required_row(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<JobRecord, JobError> {
    let row = optional_row(tx, id)
        .await?
        .ok_or_else(|| JobError::Missing(id.into()))?;
    decode_job(&row)
}

pub(super) fn decode_job(row: &SqliteRow) -> Result<JobRecord, JobError> {
    let payload: Option<&str> = row.try_get("payload")?;
    let result: Option<&str> = row.try_get("result")?;
    Ok(JobRecord {
        job_id: row.try_get("job_id")?,
        kind: row.try_get("kind")?,
        state: JobState::parse(row.try_get("state")?)?,
        attempt: row.try_get("attempt")?,
        max_attempts: row.try_get("max_attempts")?,
        available_at: row.try_get("available_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        lease_owner: row.try_get("lease_owner")?,
        lease_until: row.try_get("lease_until")?,
        cancel_requested: row.try_get("cancel_requested")?,
        last_error: row.try_get("last_error")?,
        payload: payload.map(serde_json::from_str).transpose()?,
        result: result.map(serde_json::from_str).transpose()?,
    })
}
