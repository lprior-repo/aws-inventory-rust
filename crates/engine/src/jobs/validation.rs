//! Pure bounded queue input parsing and JSON encoding.
use super::*;

pub(super) fn validate_spec(spec: &JobSpec, now: i64) -> Result<(), JobError> {
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

pub(super) fn validate_key(value: &str, label: &'static str) -> Result<(), JobError> {
    validate_identifier(value, label)?;
    if value.len() > 256 {
        return Err(JobError::Invalid(
            "job identifiers must be at most 256 bytes",
        ));
    }
    Ok(())
}

pub(super) fn check_time(now: i64) -> Result<(), JobError> {
    if now < 0 {
        return Err(JobError::Invalid("negative timestamp"));
    }
    Ok(())
}

pub(super) fn lease_end(now: i64, duration: i64) -> Result<i64, JobError> {
    check_time(now)?;
    if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&duration) {
        return Err(JobError::Invalid("lease must be 10..3600 seconds"));
    }
    now.checked_add(duration)
        .ok_or(JobError::Invalid("lease timestamp overflow"))
}

pub(super) fn json_text(value: &Value, limit: usize) -> Result<String, JobError> {
    validate_serialized_size(value, limit, "job JSON bytes")?;
    Ok(serde_json::to_string(value)?)
}

pub(super) fn bounded_error(value: String) -> String {
    let mut used = 0_usize;
    value
        .chars()
        .take_while(|character| {
            used = used.saturating_add(character.len_utf8());
            used <= MAX_ERROR_BYTES
        })
        .collect()
}
