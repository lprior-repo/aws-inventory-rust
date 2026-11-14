//! Operator cancellation and bounded retry transitions.
use super::*;

impl JobStore {
    /// Request cancellation using nonnegative millisecond `now`.
    /// Queued jobs become cancelled immediately; running jobs retain their lease
    /// with the cancellation flag set. Already terminal jobs are unchanged.
    /// # Errors
    /// Returns invalid-key/time, missing-job, SQL or persisted-row decoding errors.
    #[tracing::instrument(skip(self), fields(job_id))]
    pub async fn cancel(&self, job_id: &str, now: i64) -> Result<JobRecord, JobError> {
        validate_key(job_id, "job ID")?;
        check_time(now)?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query("UPDATE inventory_jobs SET cancel_requested=1,state=CASE WHEN state='queued' THEN 'cancelled' ELSE state END,updated_at=max(updated_at,?1) WHERE job_id=?2 AND state IN ('queued','running')")
            .bind(now).bind(job_id).execute(&mut *tx).await?;
        let record = required_row(&mut tx, job_id).await?;
        tx.commit().await?;
        Ok(record)
    }

    /// Operator-authorized retry retains all attempts and requeues eligible terminal work.
    /// Sets the total attempt budget to claims already started plus `additional`, capped at 32.
    /// Clears terminal output and cancellation, and schedules at nonnegative millisecond `now`.
    /// # Errors
    /// Accepts failed, partial or cancelled jobs; rejects other states and invalid attempt budgets.
    /// Also rejects invalid key/time, missing jobs or SQL/persisted-row decoding failures.
    #[tracing::instrument(skip(self), fields(job_id))]
    pub async fn retry(
        &self,
        job_id: &str,
        additional: u32,
        now: i64,
    ) -> Result<JobRecord, JobError> {
        validate_key(job_id, "job ID")?;
        check_time(now)?;
        if !(1..=MAX_JOB_ATTEMPTS).contains(&additional) {
            return Err(JobError::Invalid("additional attempts must be 1..32"));
        }
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let record = required_row(&mut tx, job_id).await?;
        if matches!(
            record.state,
            JobState::Queued | JobState::Running | JobState::Succeeded
        ) {
            return Err(JobError::Invalid(
                "only failed, partial or cancelled jobs can be retried",
            ));
        }
        let max = record
            .attempt
            .checked_add(additional)
            .filter(|n| *n <= MAX_JOB_ATTEMPTS)
            .ok_or(JobError::Limit("32 total attempts"))?;
        sqlx::query("UPDATE inventory_jobs SET state='queued',cancel_requested=0,max_attempts=?1,available_at=?2,updated_at=max(updated_at,?2),result=NULL,last_error=NULL WHERE job_id=?3")
            .bind(max).bind(now).bind(job_id).execute(&mut *tx).await?;
        let record = required_row(&mut tx, job_id).await?;
        tx.commit().await?;
        Ok(record)
    }
}
