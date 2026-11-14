use super::*;

pub(super) struct Outcomes {
    rows: Vec<Value>,
    bytes: usize,
    intake: Intake,
}

impl Outcomes {
    pub(super) fn new(limit: usize) -> Result<Self> {
        let mut rows = Vec::new();
        rows.try_reserve(limit)
            .context("reserve worker summaries")?;
        Ok(Self {
            rows,
            bytes: 0,
            intake: Intake::Accepting,
        })
    }

    pub(super) fn accepting(&self) -> bool {
        matches!(self.intake, Intake::Accepting)
    }

    pub(super) fn stop(&mut self, reason: StopReason) {
        self.intake = Intake::Stopped(reason);
    }

    pub(super) fn cancelled_job(&self) -> bool {
        matches!(self.intake, Intake::Stopped(StopReason::CancelledJob))
    }

    pub(super) fn stop_reason(&self) -> &'static str {
        match self.intake {
            Intake::Accepting => "claim_limit",
            Intake::Stopped(reason) => reason.as_str(),
        }
    }

    pub(super) fn record(&mut self, outcome: Value) -> Result<()> {
        inventory_core::validate_serialized_size(&outcome, MAX_SUMMARY_BYTES, "job summary")?;
        let bytes = serde_json::to_vec(&outcome)?.len();
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .context("worker result byte count overflow")?;
        ensure!(
            self.bytes <= MAX_WORKER_OUTPUT_BYTES.saturating_sub(8192),
            "worker result byte budget exceeded; complete results remain in the queue"
        );
        if outcome.get("state").and_then(Value::as_str) == Some("cancelled") {
            self.stop(StopReason::CancelledJob);
        }
        self.rows.push(outcome);
        Ok(())
    }

    pub(super) fn report(self, owner: &str) -> Result<Value> {
        let cancelled = matches!(
            self.intake,
            Intake::Stopped(StopReason::CancelledJob | StopReason::WorkerShutdown)
        );
        let partial = self
            .rows
            .iter()
            .any(|row| row.get("state").and_then(Value::as_str) != Some("succeeded"));
        let state = if cancelled {
            "cancelled"
        } else if partial {
            "partial"
        } else {
            "complete"
        };
        let value = json!({"status":state,"worker":owner,"executed":self.rows.len(),"jobs":self.rows,
            "stop_reason":self.stop_reason(),
            "mode":"bounded worker invocation; no background daemon remains",
            "results":"Terminal results are retained by job ID in the queue; oversized results declare result_truncated. This response contains bounded summaries."});
        inventory_core::validate_serialized_size(&value, MAX_WORKER_OUTPUT_BYTES, "worker output")?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Cancellation {
    Requested,
    NotRequested,
}

impl From<bool> for Cancellation {
    fn from(requested: bool) -> Self {
        if requested {
            Self::Requested
        } else {
            Self::NotRequested
        }
    }
}

pub(super) fn completion_state(value: &Value, cancellation: Cancellation) -> TerminalJobState {
    match cancellation {
        Cancellation::Requested => TerminalJobState::Cancelled,
        Cancellation::NotRequested => match value.get("status").and_then(Value::as_str) {
            Some("cancelled") => TerminalJobState::Cancelled,
            Some("complete") => TerminalJobState::Succeeded,
            _ => TerminalJobState::Partial,
        },
    }
}

pub(super) fn job_summary(job: &JobRecord) -> Value {
    let error = job
        .last_error
        .as_ref()
        .map(|value| value.chars().take(256).collect::<String>());
    let error_truncated = job
        .last_error
        .as_ref()
        .zip(error.as_ref())
        .is_some_and(|(full, brief)| full.len() > brief.len());
    let scan_count = job.result.as_ref().and_then(|value| {
        value.get("scan_count").and_then(Value::as_u64).or_else(|| {
            value
                .get("scans")
                .and_then(Value::as_array)
                .and_then(|scans| u64::try_from(scans.len()).ok())
        })
    });
    let result_truncated = job
        .result
        .as_ref()
        .and_then(|value| value.get("result_truncated"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    json!({"job_id":job.job_id,"state":job.state,"attempt":job.attempt,
        "scan_count":scan_count,"result_available":job.result.is_some(),"error":error,
        "error_truncated":error_truncated,"result_truncated":result_truncated})
}

pub(super) fn compact_result(value: Value) -> Value {
    if inventory_core::validate_serialized_size(
        &value,
        inventory_engine::jobs::MAX_RESULT_BYTES,
        "job result",
    )
    .is_ok()
    {
        return value;
    }
    let scan_count = value
        .get("scans")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let scans: Vec<Value> = value
        .get("scans")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(crate::targets::MAX_TARGET_ACCOUNTS)
        .map(compact_scan)
        .collect();
    let status = bounded_text(value.get("status"), 32);
    let compact = json!({"status":status,"scans":scans,"scan_count":scan_count,
        "result_truncated":true,"details":"Full observations and scope failures remain in the inventory database."});
    if inventory_core::validate_serialized_size(
        &compact,
        inventory_engine::jobs::MAX_RESULT_BYTES,
        "compacted job result",
    )
    .is_ok()
    {
        return compact;
    }
    json!({"status":status,"scans":[],"scan_count":scan_count,
        "result_truncated":true,"summaries_omitted":true,
        "details":"Terminal summaries exceeded the result budget. Observations and scope failures remain in the inventory database."})
}

pub(super) fn compact_scan(scan: &Value) -> Value {
    let count = |key: &str| {
        scan.get("summary")
            .and_then(|summary| summary.get(key))
            .and_then(Value::as_u64)
    };
    json!({"scan_id":bounded_text(scan.get("scan_id"),256),
    "status":bounded_text(scan.get("status"),32),"summary":{
        "resources":count("resources"),"edges":count("edges"),
        "completed_scopes":count("completed_scopes"),"failed_scopes":count("failed_scopes")
    }})
}

pub(super) fn bounded_text(value: Option<&Value>, bytes: usize) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .filter(|value| value.len() <= bytes)
}

enum Intake {
    Accepting,
    Stopped(StopReason),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum StopReason {
    CancelledJob,
    WorkerShutdown,
    QueueEmpty,
}

impl StopReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::CancelledJob => "cancelled_job",
            Self::WorkerShutdown => "worker_shutdown",
            Self::QueueEmpty => "queue_empty",
        }
    }
}
