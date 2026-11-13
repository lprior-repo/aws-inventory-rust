//! validation operations.
use super::*;

pub(super) fn validate_account_id(account: &str) -> Result<(), AwsScanError> {
    if account.len() != 12 || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return invalid("AWS account ID must have exactly 12 decimal digits");
    }
    Ok(())
}

pub(super) fn validate_partition(partition: &str) -> Result<(), AwsScanError> {
    if !(3..=32).contains(&partition.len())
        || !partition.split('-').enumerate().all(|(position, segment)| {
            if position == 0 {
                segment == "aws"
            } else {
                !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_lowercase())
            }
        })
    {
        return invalid("invalid AWS partition");
    }
    Ok(())
}

pub(super) fn bounded_text(value: &str, label: &str, limit: usize) -> Result<(), AwsScanError> {
    if value.is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(data_error(&format!("invalid or oversized {label}")));
    }
    Ok(())
}

pub(super) fn optional_text(
    value: Option<&str>,
    label: &str,
    limit: usize,
) -> Result<Option<String>, AwsScanError> {
    value
        .map(|value| {
            bounded_text(value, label, limit)?;
            Ok(value.to_owned())
        })
        .transpose()
}

pub(super) fn data_error(message: &str) -> AwsScanError {
    AwsScanError::InvalidData(message.to_owned())
}

pub(super) fn invalid<T>(message: &str) -> Result<T, AwsScanError> {
    Err(AwsScanError::InvalidOptions(message.to_owned()))
}

pub(super) fn ensure_running(executor: &ApiExecutor) -> Result<(), AwsScanError> {
    if executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    Ok(())
}
