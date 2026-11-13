//! Validate bounded operation status histograms.

use crate::HarnessError;

use super::Json;

fn is_status_code(code: &str) -> bool {
    code == "None"
        || (!code.is_empty()
            && code.chars().all(|value| value.is_ascii_digit())
            && code
                .parse::<i64>()
                .map(|value| (100..=599).contains(&value))
                .unwrap_or(false))
}

/// Validate and copy one operation status histogram.
pub fn checked_histogram(value: &Json) -> Result<Json, HarnessError> {
    let entries = value
        .as_object()
        .ok_or_else(|| HarnessError::invalid("invalid operation status histogram"))?;
    if entries.len() > 600 {
        return Err(HarnessError::invalid("invalid operation status histogram"));
    }
    for (code, count) in entries {
        if !is_status_code(code) || count.as_i64().map(|value| value > 0) != Some(true) {
            return Err(HarnessError::invalid(
                "invalid operation status histogram entry",
            ));
        }
    }
    Ok(value.clone())
}
