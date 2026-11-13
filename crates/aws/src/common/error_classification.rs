//! error classification operations.
use super::*;

pub(super) fn classify_sdk_error<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
) -> ApiError {
    let status = error
        .raw_response()
        .map(|response| response.status().as_u16());
    if let Some(service) = error.as_service_error() {
        if status.is_some_and(|status| (200..300).contains(&status))
            && ProvideErrorMetadata::code(service).is_none()
        {
            return ApiError::Permanent(
                "SDK rejected an HTTP success response without a modeled error code; check response decoding and protocol compatibility".into(),
            );
        }
        let code = ProvideErrorMetadata::code(service).unwrap_or("UnknownServiceError");
        let detail = format!(
            "{}: {}",
            code,
            service.message().unwrap_or("no service message")
        );
        let detail: String = detail.chars().take(1_024).collect();
        let normalized = code.to_ascii_lowercase();
        if status == Some(501) {
            return ApiError::Permanent(detail);
        }
        if matches!(
            service.retryable_error_kind(),
            Some(ErrorKind::ThrottlingError)
        ) || status == Some(429)
            || normalized.contains("throttl")
            || normalized.contains("toomanyrequests")
            || normalized.contains("requestlimitexceeded")
            || normalized.contains("slowdown")
        {
            return ApiError::Throttled(detail);
        }
        if status == Some(408) || normalized.contains("requesttimeout") {
            return ApiError::Timeout;
        }
        if matches!(status, Some(401 | 403))
            || normalized.contains("accessdenied")
            || normalized.contains("unauthorized")
            || normalized.contains("unrecognizedclient")
            || normalized.contains("expiredtoken")
            || normalized.contains("invalidclienttoken")
            || normalized.contains("authfailure")
        {
            return ApiError::Denied(detail);
        }
        if matches!(
            service.retryable_error_kind(),
            Some(ErrorKind::ServerError | ErrorKind::TransientError)
        ) || status.is_some_and(|status| status >= 500)
            || normalized.contains("serviceunavailable")
            || normalized.contains("internal")
        {
            return ApiError::Transient(detail);
        }
        return ApiError::Permanent(detail);
    }
    if status == Some(429) {
        return ApiError::Throttled("HTTP 429 without a modeled service error".to_owned());
    }
    if status == Some(408) {
        return ApiError::Timeout;
    }
    if matches!(status, Some(401 | 403)) {
        return ApiError::Denied(
            "HTTP authentication or authorization failure without a modeled service error".into(),
        );
    }
    if status == Some(501) {
        return ApiError::Permanent("HTTP 501 Not Implemented".into());
    }
    match error {
        SdkError::TimeoutError(_) => ApiError::Timeout,
        SdkError::DispatchFailure(_) | SdkError::ResponseError(_) => {
            ApiError::Transient("SDK transport or response failure".to_owned())
        }
        _ => ApiError::Permanent("SDK request construction failure".to_owned()),
    }
}

pub(crate) fn error_category(error: &AwsScanError) -> &'static str {
    match error {
        AwsScanError::Api(ApiError::Denied(_)) => "access_denied",
        AwsScanError::Api(ApiError::Cancelled) => "cancelled",
        AwsScanError::Api(ApiError::Throttled(_)) => "throttled",
        AwsScanError::Api(ApiError::RetryAfter {
            kind: RetryKind::Throttled,
            ..
        }) => "throttled",
        AwsScanError::Api(ApiError::RetryAfter {
            kind: RetryKind::Timeout,
            ..
        }) => "timeout",
        AwsScanError::Api(ApiError::RetryAfterRejected(_)) => "retry_after_rejected",
        AwsScanError::Api(ApiError::Timeout) => "timeout",
        AwsScanError::Api(ApiError::Exhausted(_)) => "retry_budget_exhausted",
        AwsScanError::Api(ApiError::UnsupportedType(_)) => "unsupported_type",
        AwsScanError::Api(_) => "api_error",
        AwsScanError::InvalidData(_) | AwsScanError::Validation(_) => "invalid_response",
        AwsScanError::InvalidOptions(_) => "invalid_options",
        AwsScanError::ChannelClosed => "consumer_closed",
        AwsScanError::ChildLimit => "child_limit",
    }
}
