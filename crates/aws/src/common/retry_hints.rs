use super::*;

pub(crate) fn sdk_error<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
) -> ApiError {
    sdk_error_with_hint(error, None)
}

pub(crate) fn sdk_error_with_hint<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
    typed_hint: Option<RetryAfter>,
) -> ApiError {
    let now = SystemTime::now();
    let raw_hint = error.raw_response().and_then(|response| {
        let standard = header_hints(response.headers().get_all_bytes("retry-after"), false, now);
        let aws = header_hints(
            response.headers().get_all_bytes("x-amz-retry-after"),
            true,
            now,
        );
        combine_hints(standard, aws)
    });
    attach_hint(
        classify_sdk_error(error),
        combine_hints(raw_hint, typed_hint),
    )
}

pub(super) fn attach_hint(error: ApiError, hint: Option<RetryAfter>) -> ApiError {
    let Some(retry_after) = hint else {
        return error;
    };
    let (kind, message) = match error {
        ApiError::Throttled(message) => (RetryKind::Throttled, message),
        ApiError::Transient(message) => (RetryKind::Transient, message),
        ApiError::Timeout => (RetryKind::Timeout, "HTTP or SDK attempt timeout".into()),
        other => return other,
    };
    ApiError::RetryAfter {
        kind,
        message,
        retry_after,
    }
}

pub(super) fn combine_hints(
    left: Option<RetryAfter>,
    right: Option<RetryAfter>,
) -> Option<RetryAfter> {
    match (left, right) {
        (Some(RetryAfter::Excessive), _) | (_, Some(RetryAfter::Excessive)) => {
            Some(RetryAfter::Excessive)
        }
        (Some(RetryAfter::Invalid), _) | (_, Some(RetryAfter::Invalid)) => {
            Some(RetryAfter::Invalid)
        }
        (Some(RetryAfter::Delay(left)), Some(RetryAfter::Delay(right))) => {
            Some(RetryAfter::Delay(left.max(right)))
        }
        (left, None) => left,
        (None, right) => right,
    }
}

pub(super) fn header_hints<'a>(
    values: impl Iterator<Item = &'a [u8]>,
    milliseconds: bool,
    now: SystemTime,
) -> Option<RetryAfter> {
    let mut result = None;
    for (index, bytes) in values
        .take(MAX_RETRY_HEADER_VALUES.saturating_add(1))
        .enumerate()
    {
        if index == MAX_RETRY_HEADER_VALUES {
            return Some(RetryAfter::Invalid);
        }
        let hint = match bounded_hint_text(bytes) {
            Some(value) if milliseconds => parse_delay_integer(value, 1),
            Some(value) => parse_standard_retry_after(value, now),
            None => RetryAfter::Invalid,
        };
        result = combine_hints(result, Some(hint));
    }
    result
}

pub(super) fn bounded_hint_text(bytes: &[u8]) -> Option<&str> {
    if bytes.len() > MAX_RETRY_HINT_BYTES {
        return None;
    }
    std::str::from_utf8(bytes)
        .ok()
        .map(|value| value.trim_matches([' ', '\t']))
        .filter(|value| !value.is_empty())
}

pub(crate) fn parse_retry_after_seconds(value: &str) -> RetryAfter {
    match bounded_hint_text(value.as_bytes()) {
        Some(value) => parse_delay_integer(value, 1_000),
        None => RetryAfter::Invalid,
    }
}

pub(super) fn parse_delay_integer(value: &str, milliseconds_per_unit: u64) -> RetryAfter {
    if value.is_empty()
        || value.len() > MAX_RETRY_HINT_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return RetryAfter::Invalid;
    }
    let milliseconds = value
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(milliseconds_per_unit));
    match milliseconds {
        Some(value) if value <= MAX_RETRY_AFTER_MS => {
            RetryAfter::Delay(Duration::from_millis(value))
        }
        Some(_) | None => RetryAfter::Excessive,
    }
}

pub(super) fn parse_standard_retry_after(value: &str, now: SystemTime) -> RetryAfter {
    if value.is_empty() || value.len() > MAX_RETRY_HINT_BYTES {
        return RetryAfter::Invalid;
    }
    if value.bytes().all(|byte| byte.is_ascii_digit()) {
        return parse_delay_integer(value, 1_000);
    }
    let date = match DateTime::from_str(value, Format::HttpDate) {
        Ok(date) => date,
        Err(_) => return RetryAfter::Invalid,
    };
    let deadline = match SystemTime::try_from(date) {
        Ok(deadline) => deadline,
        Err(_) => return RetryAfter::Excessive,
    };
    let delay = deadline.duration_since(now).unwrap_or_default();
    if delay <= Duration::from_millis(MAX_RETRY_AFTER_MS) {
        RetryAfter::Delay(delay)
    } else {
        RetryAfter::Excessive
    }
}
