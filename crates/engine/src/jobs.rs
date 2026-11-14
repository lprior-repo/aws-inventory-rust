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

pub(crate) const APPLICATION_ID: i64 = 1_095_325_521;
pub(crate) const MAX_JOBS: i64 = 10_000;
pub(crate) const MAX_JOB_ATTEMPTS: u32 = 32;
pub(crate) const MAX_PAYLOAD_BYTES: usize = 65_536;
/// Maximum compact-JSON bytes retained as one attempt's result (1 MiB).
pub const MAX_RESULT_BYTES: usize = 1_048_576;
pub(crate) const MAX_LIST_ROWS: usize = 1_000;
const MAX_ERROR_BYTES: usize = 8_192;
const MIN_LEASE_MS: i64 = 10_000;
const MAX_LEASE_MS: i64 = 3_600_000;

/// Queue storage, input, identity-conflict and fenced-lease failures.
#[derive(Debug, Error)]
pub enum JobError {
    /// SQLite access or stored-row decoding failed.
    #[error("job database: {0}")]
    Database(#[from] sqlx::Error),
    /// Job payload/result serialization or stored JSON decoding failed.
    #[error("job JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Identifier or serialized-size validation failed.
    #[error("job validation: {0}")]
    Validation(#[from] ValidationError),
    /// Invalid identifier length, timestamp, attempt budget or state transition.
    #[error("invalid job input: {0}")]
    Invalid(&'static str),
    /// Retained jobs, total attempts or serialized output exceeded a bound.
    #[error("job limit exceeded: {0}")]
    Limit(&'static str),
    /// No retained job has the requested ID.
    #[error("job does not exist: {0}")]
    Missing(String),
    /// An existing ID has a different immutable kind, payload or initial budget.
    #[error("job ID already identifies a different immutable specification")]
    Conflict,
    /// The attempt expired or no longer matches the lease's owner/token fence.
    #[error("job lease is expired or belongs to another attempt")]
    LostLease,
    /// The SQLite application identity or queue schema version is unsupported.
    #[error("database belongs to a different application or unsupported queue version")]
    ForeignDatabase,
}

/// Persisted lifecycle state; only queued jobs may be newly claimed.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Eligible once its availability time arrives and attempt budget remains.
    Queued,
    /// Claimed by one worker under an expiring, fenced lease.
    Running,
    /// Worker acknowledged successful completion.
    Succeeded,
    /// Worker acknowledged an incomplete result.
    Partial,
    /// Worker acknowledged failure, or an expired attempt exhausted its budget.
    Failed,
    /// Cancellation was acknowledged or queued work was cancelled immediately.
    Cancelled,
}

impl JobState {
    /// Lowercase spelling persisted in the queue and serialized in job reports.
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

/// Immutable submission identity and initial scheduling policy.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    /// Nonempty validated ID of at most 256 bytes; repeated submissions are fenced by it.
    pub job_id: String,
    /// Validated dispatch kind of at most 64 bytes; not interpreted by the queue.
    pub kind: String,
    /// JSON object payload with at most 65,536 serialized bytes.
    pub payload: Value,
    /// Initial total claim budget in `1..=32`, including the first attempt.
    pub max_attempts: u32,
    /// Nonnegative earliest claim time, in caller-supplied milliseconds.
    pub available_at: i64,
}

/// Current retained job state; list responses omit payload and result values.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobRecord {
    /// Immutable submission identity.
    pub job_id: String,
    /// Immutable worker-dispatch kind.
    pub kind: String,
    /// Current persisted lifecycle outcome.
    pub state: JobState,
    /// Claims started so far; zero means never claimed.
    pub attempt: u32,
    /// Current total claim budget, including operator-authorized retries.
    pub max_attempts: u32,
    /// Earliest eligible claim time in caller-supplied milliseconds.
    pub available_at: i64,
    /// Submission time in caller-supplied milliseconds.
    pub created_at: i64,
    /// Monotonic retained update time in caller-supplied milliseconds.
    pub updated_at: i64,
    /// Current worker identity, or `None` when no running lease is retained.
    pub lease_owner: Option<String>,
    /// Lease expiry in caller-supplied milliseconds, or `None` without a lease.
    pub lease_until: Option<i64>,
    /// A running worker must stop and acknowledge cancellation when this is set.
    pub cancel_requested: bool,
    /// Bounded diagnostic from the last attempt, or `None` if absent/reset.
    pub last_error: Option<String>,
    /// Submission object; `None` in list responses to avoid materializing payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    /// Retained completion output; `None` if absent, reset or omitted by listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Claim capability fenced by private job, attempt, owner and token values.
#[derive(Debug)]
pub struct Lease {
    /// Claimed job snapshot; changing it does not change the private lease fence.
    pub job: JobRecord,
    job_id: String,
    attempt: u32,
    owner: String,
    token: String,
}

/// Retained attempt history; heartbeats update the job but add no history rows.
#[derive(Clone, Debug, Serialize)]
pub struct AttemptRecord {
    /// One-based claim number, at most 32 per job.
    pub attempt: u32,
    /// Worker identity recorded when the claim began.
    pub worker: String,
    /// Claim start time in caller-supplied milliseconds.
    pub started_at: i64,
    /// Terminal time in milliseconds; `None` while this attempt is running.
    pub finished_at: Option<i64>,
    /// Stored spelling, including `running`, terminal states or `expired`.
    pub state: String,
    /// Bounded terminal diagnostic, or `None` when no diagnostic was retained.
    pub error: Option<String>,
}

/// A terminal acknowledgement cannot enqueue or start another attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalJobState {
    /// Successful terminal acknowledgement; cannot schedule another attempt.
    Succeeded,
    /// Terminal acknowledgement retaining incomplete results.
    Partial,
    /// Terminal failed acknowledgement, independent of diagnostic presence.
    Failed,
    /// Terminal cancellation acknowledgement, independent of diagnostic presence.
    Cancelled,
}

impl TerminalJobState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<JobState> for TerminalJobState {
    type Error = JobError;

    fn try_from(state: JobState) -> Result<Self, Self::Error> {
        match state {
            JobState::Succeeded => Ok(Self::Succeeded),
            JobState::Partial => Ok(Self::Partial),
            JobState::Failed => Ok(Self::Failed),
            JobState::Cancelled => Ok(Self::Cancelled),
            JobState::Queued | JobState::Running => {
                Err(JobError::Invalid("completion must be terminal"))
            }
        }
    }
}

/// Terminal acknowledgement data; pending cancellation overrides the supplied state.
#[derive(Debug)]
pub struct Completion {
    /// Requested terminal outcome, superseded by a persisted cancellation request.
    pub state: TerminalJobState,
    /// Retained diagnostic output is independent of the terminal outcome.
    /// Failed and cancelled attempts may still have useful partial results.
    pub result: Option<Value>,
    /// Diagnostics may be absent for any terminal outcome; this is not a state flag.
    pub error: Option<String>,
}

/// Single-connection durable job queue with transactional claims and bounded history.
pub struct JobStore {
    pool: SqlitePool,
}

impl JobStore {}

mod connection;
mod execution;
mod read;
mod transitions;
use read::*;
mod validation;
use validation::*;
