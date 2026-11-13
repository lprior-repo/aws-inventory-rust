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

/// A terminal acknowledgement cannot enqueue or start another attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalJobState {
    Succeeded,
    Partial,
    Failed,
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

#[derive(Debug)]
pub struct Completion {
    pub state: TerminalJobState,
    /// Retained diagnostic output is independent of the terminal outcome.
    /// Failed and cancelled attempts may still have useful partial results.
    pub result: Option<Value>,
    /// Diagnostics may be absent for any terminal outcome; this is not a state flag.
    pub error: Option<String>,
}

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
