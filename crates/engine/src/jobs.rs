//! A bounded SQLx scan-job queue with transactional claims and fenced leases.
//!
//! Time and lease tokens are caller capabilities. Expired work is retried as a
//! new attempt, never as a claim that mutable AWS pagination can be replayed.
//! Heartbeats do not append history; retained history is bounded by jobs*32.
use std::path::Path;
use std::time::Duration;

use inventory_core::{ValidationError, validate_identifier, validate_serialized_size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous,
};
use sqlx::{ConnectOptions, Row, Sqlite, SqlitePool, Transaction};
use thiserror::Error;

pub const APPLICATION_ID: i64 = 1_095_325_521;
pub const MAX_JOBS: i64 = 10_000;
pub const MAX_JOB_ATTEMPTS: u32 = 32;
pub const MAX_PAYLOAD_BYTES: usize = 65_536;
pub const MAX_RESULT_BYTES: usize = 1_048_576;
pub const MAX_LIST_ROWS: usize = 1_000;
const MAX_ERROR_BYTES: usize = 8_192;
const MIN_LEASE_MS: i64 = 10_000;
const MAX_LEASE_MS: i64 = 3_600_000;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("job database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("job JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("job validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("invalid job input: {0}")]
    Invalid(&'static str),
    #[error("job limit exceeded: {0}")]
    Limit(&'static str),
    #[error("job does not exist: {0}")]
    Missing(String),
    #[error("job ID already identifies a different immutable specification")]
    Conflict,
    #[error("job lease is expired or belongs to another attempt")]
    LostLease,
    #[error("database belongs to a different application or unsupported queue version")]
    ForeignDatabase,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
}

impl JobState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    fn parse(value: &str) -> Result<Self, JobError> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "partial" => Ok(Self::Partial),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(JobError::Invalid("unknown stored state")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    pub job_id: String,
    pub kind: String,
    pub payload: Value,
    pub max_attempts: u32,
    pub available_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobRecord {
    pub job_id: String,
    pub kind: String,
    pub state: JobState,
    pub attempt: u32,
    pub max_attempts: u32,
    pub available_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub lease_owner: Option<String>,
    pub lease_until: Option<i64>,
    pub cancel_requested: bool,
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

#[derive(Debug)]
pub struct Lease {
    pub job: JobRecord,
    job_id: String,
    attempt: u32,
    owner: String,
    token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttemptRecord {
    pub attempt: u32,
    pub worker: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct Completion {
    pub state: JobState,
    pub result: Option<Value>,
    pub error: Option<String>,
}

pub struct JobStore {
    pool: SqlitePool,
}

impl JobStore {
    #[tracing::instrument(skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, JobError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(JobError::Invalid("empty queue path"));
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .in_memory(path == Path::new(":memory:"))
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await?;
        if let Err(error) = initialize(&pool).await {
            pool.close().await;
            return Err(error);
        }
        Ok(Self { pool })
    }

    #[tracing::instrument(skip(self))]
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// The first submission owns scheduling time. Repeated identical IDs return
    /// that immutable job; a different payload/kind/initial attempt budget fails.
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

    #[tracing::instrument(skip_all, fields(job_id = %lease.job_id, attempt = lease.attempt))]
    pub async fn complete(
        &self,
        lease: &Lease,
        completion: Completion,
        now: i64,
    ) -> Result<JobRecord, JobError> {
        check_time(now)?;
        if matches!(completion.state, JobState::Queued | JobState::Running) {
            return Err(JobError::Invalid("completion must be terminal"));
        }
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

    /// Operator-authorized retry adds a finite budget and retains all attempts.
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

    /// Listing omits potentially large payload/result JSON. Use get for one job.
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

#[tracing::instrument(skip(pool))]
async fn initialize(pool: &SqlitePool) -> Result<(), JobError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let application: i64 = sqlx::query_scalar("PRAGMA application_id")
        .fetch_one(&mut *tx)
        .await?;
    let tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(&mut *tx)
    .await?;
    if application == 0 && tables == 0 {
        sqlx::raw_sql(include_str!("../sql/jobs.sql"))
            .execute(&mut *tx)
            .await?;
    } else if application == APPLICATION_ID {
        let version: i64 =
            sqlx::query_scalar("SELECT version FROM inventory_job_schema WHERE singleton=1")
                .fetch_one(&mut *tx)
                .await?;
        if version != 1 {
            return Err(JobError::ForeignDatabase);
        }
    } else {
        return Err(JobError::ForeignDatabase);
    }
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn recover_expired(tx: &mut Transaction<'_, Sqlite>, now: i64) -> Result<(), JobError> {
    sqlx::query("UPDATE inventory_job_attempts SET state='expired',finished_at=max(started_at,?1),error='worker lease expired before terminal acknowledgement' WHERE state='running' AND EXISTS (SELECT 1 FROM inventory_jobs j WHERE j.job_id=inventory_job_attempts.job_id AND j.attempt=inventory_job_attempts.attempt AND j.state='running' AND j.lease_until<=?1)")
        .bind(now).execute(&mut **tx).await?;
    sqlx::query("UPDATE inventory_jobs SET state=CASE WHEN cancel_requested=1 THEN 'cancelled' WHEN attempt>=max_attempts THEN 'failed' ELSE 'queued' END,lease_owner=NULL,lease_token=NULL,lease_until=NULL,available_at=max(available_at,?1),updated_at=max(updated_at,?1),last_error='worker lease expired before terminal acknowledgement' WHERE state='running' AND lease_until<=?1")
        .bind(now).execute(&mut **tx).await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn optional_row(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> Result<Option<SqliteRow>, JobError> {
    Ok(sqlx::query("SELECT * FROM inventory_jobs WHERE job_id=?1")
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?)
}

#[tracing::instrument(skip_all)]
async fn required_row(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<JobRecord, JobError> {
    let row = optional_row(tx, id)
        .await?
        .ok_or_else(|| JobError::Missing(id.into()))?;
    decode_job(&row)
}

fn decode_job(row: &SqliteRow) -> Result<JobRecord, JobError> {
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

fn validate_spec(spec: &JobSpec, now: i64) -> Result<(), JobError> {
    validate_key(&spec.job_id, "job ID")?;
    validate_key(&spec.kind, "job kind")?;
    check_time(now)?;
    check_time(spec.available_at)?;
    if spec.kind.len() > 64 || !spec.payload.is_object() {
        return Err(JobError::Invalid("job kind or object payload"));
    }
    if !(1..=MAX_JOB_ATTEMPTS).contains(&spec.max_attempts) {
        return Err(JobError::Invalid("attempt budget must be 1..32"));
    }
    Ok(())
}

fn validate_key(value: &str, label: &'static str) -> Result<(), JobError> {
    validate_identifier(value, label)?;
    if value.len() > 256 {
        return Err(JobError::Invalid(
            "job identifiers must be at most 256 bytes",
        ));
    }
    Ok(())
}

fn check_time(now: i64) -> Result<(), JobError> {
    if now < 0 {
        return Err(JobError::Invalid("negative timestamp"));
    }
    Ok(())
}

fn lease_end(now: i64, duration: i64) -> Result<i64, JobError> {
    check_time(now)?;
    if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&duration) {
        return Err(JobError::Invalid("lease must be 10..3600 seconds"));
    }
    now.checked_add(duration)
        .ok_or(JobError::Invalid("lease timestamp overflow"))
}

fn json_text(value: &Value, limit: usize) -> Result<String, JobError> {
    validate_serialized_size(value, limit, "job JSON bytes")?;
    Ok(serde_json::to_string(value)?)
}

fn bounded_error(value: String) -> String {
    let mut used = 0_usize;
    value
        .chars()
        .take_while(|character| {
            used = used.saturating_add(character.len_utf8());
            used <= MAX_ERROR_BYTES
        })
        .collect()
}
