//! Transactional submission, fenced claims, and terminal acknowledgement.
use super::*;

impl JobStore {
    /// The first submission owns scheduling time. Repeated identical IDs return
    /// that immutable job; a different payload/kind/initial attempt budget fails.
    /// `now` is nonnegative caller time in milliseconds; the persisted row is returned.
    /// # Errors
    /// Rejects invalid/oversized specifications, conflicting replays, the 10,000-job
    /// capacity bound, or JSON/SQL/persisted-row decoding failures.
    #[tracing::instrument(skip_all, fields(job_id = %spec.job_id))]
    pub async fn submit(&self, spec: JobSpec, now: i64) -> Result<JobRecord, JobError> {
        validate_spec(&spec, now)?;
        let payload = json_text(&spec.payload, MAX_PAYLOAD_BYTES)?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        if let Some(row) = optional_row(&mut tx, &spec.job_id).await? {
            let same = row.try_get::<&str, _>("kind")? == spec.kind
                && row.try_get::<&str, _>("payload")? == payload
                && row.try_get::<u32, _>("initial_max_attempts")? == spec.max_attempts;
            if !same {
                return Err(JobError::Conflict);
            }
            let record = decode_job(&row)?;
            tx.commit().await?;
            return Ok(record);
        }
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory_jobs")
            .fetch_one(&mut *tx)
            .await?;
        if count >= MAX_JOBS {
            return Err(JobError::Limit("10000 retained jobs"));
        }
        sqlx::query("INSERT INTO inventory_jobs(job_id,kind,payload,state,initial_max_attempts,max_attempts,available_at,created_at,updated_at) VALUES(?1,?2,?3,'queued',?4,?4,?5,?6,?6)")
            .bind(&spec.job_id).bind(&spec.kind).bind(payload).bind(spec.max_attempts)
            .bind(spec.available_at).bind(now).execute(&mut *tx).await?;
        let result = required_row(&mut tx, &spec.job_id).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Claims are serialized by SQLite, including reclaiming expired attempts.
    /// Every later write is fenced by job ID, attempt number, owner and token.
    /// `now` and `lease_ms` are milliseconds; `None` means no eligible queued job.
    /// # Errors
    /// Rejects invalid worker/token keys, negative time, leases outside 10 seconds
    /// through one hour, timestamp overflow or SQL/persisted-row decoding failures.
    #[tracing::instrument(skip_all, fields(worker = owner))]
    pub async fn claim(
        &self,
        owner: &str,
        token: &str,
        now: i64,
        lease_ms: i64,
    ) -> Result<Option<Lease>, JobError> {
        validate_key(owner, "worker")?;
        validate_key(token, "lease token")?;
        let until = lease_end(now, lease_ms)?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        recover_expired(&mut tx, now).await?;
        let job_id: Option<String> = sqlx::query_scalar("SELECT job_id FROM inventory_jobs WHERE state='queued' AND cancel_requested=0 AND attempt<max_attempts AND available_at<=?1 ORDER BY available_at,created_at,job_id LIMIT 1")
            .bind(now).fetch_optional(&mut *tx).await?;
        let Some(job_id) = job_id else {
            tx.commit().await?;
            return Ok(None);
        };
        sqlx::query("UPDATE inventory_jobs SET state='running',attempt=attempt+1,lease_owner=?1,lease_token=?2,lease_until=?3,updated_at=max(updated_at,?4),last_error=NULL,result=NULL WHERE job_id=?5 AND state='queued'")
            .bind(owner).bind(token).bind(until).bind(now).bind(&job_id).execute(&mut *tx).await?;
        let job = required_row(&mut tx, &job_id).await?;
        sqlx::query("INSERT INTO inventory_job_attempts(job_id,attempt,worker,started_at,state) VALUES(?1,?2,?3,?4,'running')")
            .bind(&job_id).bind(job.attempt).bind(owner).bind(now).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(Some(Lease {
            job_id: job.job_id.clone(),
            attempt: job.attempt,
            job,
            owner: owner.into(),
            token: token.into(),
        }))
    }

    /// A cancellation request is returned while ownership is still retained so
    /// the worker can stop intake, drain, and acknowledge cancellation.
    /// Millisecond times must remain within the live lease; returns its cancel flag.
    /// # Errors
    /// Rejects invalid lease duration/time, overflow, SQL failures or stale/fenced leases.
    #[tracing::instrument(skip_all, fields(job_id = %lease.job_id, attempt = lease.attempt))]
    pub async fn heartbeat(
        &self,
        lease: &Lease,
        now: i64,
        lease_ms: i64,
    ) -> Result<bool, JobError> {
        let until = lease_end(now, lease_ms)?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let changed = sqlx::query("UPDATE inventory_jobs SET lease_until=max(lease_until,?1),updated_at=max(updated_at,?2) WHERE job_id=?3 AND attempt=?4 AND state='running' AND lease_owner=?5 AND lease_token=?6 AND lease_until>?2")
            .bind(until).bind(now).bind(&lease.job_id).bind(lease.attempt)
            .bind(&lease.owner).bind(&lease.token).execute(&mut *tx).await?.rows_affected();
        if changed != 1 {
            return Err(JobError::LostLease);
        }
        let cancel: bool =
            sqlx::query_scalar("SELECT cancel_requested FROM inventory_jobs WHERE job_id=?1")
                .bind(&lease.job_id)
                .fetch_one(&mut *tx)
                .await?;
        tx.commit().await?;
        Ok(cancel)
    }

    /// Finish a live, exactly matching lease; a persisted cancellation wins over `state`.
    /// Millisecond `now` must precede expiry. Results are bounded to 1 MiB; error
    /// text is truncated to 8 KiB at a UTF-8 boundary.
    /// # Errors
    /// Fails on negative time, oversized/invalid JSON results, stale leases or SQL/decode errors.
    #[tracing::instrument(skip_all, fields(job_id = %lease.job_id, attempt = lease.attempt))]
    pub async fn complete(
        &self,
        lease: &Lease,
        completion: Completion,
        now: i64,
    ) -> Result<JobRecord, JobError> {
        check_time(now)?;
        let result = completion
            .result
            .as_ref()
            .map(|v| json_text(v, MAX_RESULT_BYTES))
            .transpose()?;
        let error = completion.error.map(bounded_error);
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let changed = sqlx::query("UPDATE inventory_jobs SET state=CASE WHEN cancel_requested=1 THEN 'cancelled' ELSE ?1 END,lease_owner=NULL,lease_token=NULL,lease_until=NULL,result=?2,last_error=?3,updated_at=max(updated_at,?4) WHERE job_id=?5 AND attempt=?6 AND state='running' AND lease_owner=?7 AND lease_token=?8 AND lease_until>?4")
            .bind(completion.state.as_str()).bind(result).bind(&error).bind(now)
            .bind(&lease.job_id).bind(lease.attempt).bind(&lease.owner).bind(&lease.token)
            .execute(&mut *tx).await?.rows_affected();
        if changed != 1 {
            return Err(JobError::LostLease);
        }
        let record = required_row(&mut tx, &lease.job_id).await?;
        sqlx::query("UPDATE inventory_job_attempts SET state=?1,finished_at=max(started_at,?2),error=?3 WHERE job_id=?4 AND attempt=?5 AND state='running'")
            .bind(record.state.as_str()).bind(now).bind(error).bind(&record.job_id).bind(record.attempt)
            .execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(record)
    }
}
#[tracing::instrument(skip_all)]
pub(super) async fn recover_expired(
    tx: &mut Transaction<'_, Sqlite>,
    now: i64,
) -> Result<(), JobError> {
    sqlx::query("UPDATE inventory_job_attempts SET state='expired',finished_at=max(started_at,?1),error='worker lease expired before terminal acknowledgement' WHERE state='running' AND EXISTS (SELECT 1 FROM inventory_jobs j WHERE j.job_id=inventory_job_attempts.job_id AND j.attempt=inventory_job_attempts.attempt AND j.state='running' AND j.lease_until<=?1)")
        .bind(now).execute(&mut **tx).await?;
    sqlx::query("UPDATE inventory_jobs SET state=CASE WHEN cancel_requested=1 THEN 'cancelled' WHEN attempt>=max_attempts THEN 'failed' ELSE 'queued' END,lease_owner=NULL,lease_token=NULL,lease_until=NULL,available_at=max(available_at,?1),updated_at=max(updated_at,?1),last_error='worker lease expired before terminal acknowledgement' WHERE state='running' AND lease_until<=?1")
        .bind(now).execute(&mut **tx).await?;
    Ok(())
}
