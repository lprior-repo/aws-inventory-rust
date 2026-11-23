use crate::request::Protocol;
use http::StatusCode;
use inventory_core::catalog::{ErrorKind as ModeledKind, ModeledError, Shape};

mod cbor;
mod json;
mod xml;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    Throttle,
    Retryable,
    AccessDenied,
    NotSupported,
    ExpiredToken,
    ExpiredSignature,
    Fatal,
}

#[derive(Debug, thiserror::Error)]
pub struct ServiceError {
    pub class: ErrorClass,
    pub code: Option<String>,
    pub retry_after: Option<std::time::Duration>,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AWS service failure: {:?}", self.class)?;
        if let Some(code) = &self.code {
            write!(f, " ({code})")?;
        }
        Ok(())
    }
}

pub fn decode(context: super::ResponseContext<'_>) -> Result<ServiceError, super::DecodeError> {
    if context.status.is_success() {
        return Err(super::DecodeError::Invalid(
            "success response is not a service error",
        ));
    }
    if context.body.len() > context.limits.body_bytes() {
        return Err(super::DecodeError::Limit("response body"));
    }
    let code = match error_header(context.headers)? {
        Some(code) => Some(code),
        None => body_code(context)?,
    };
    let class = match modeled_class(context, code.as_deref())? {
        Some(class) => class,
        None => match class_from_code(code.as_deref()) {
            Some(class) => class,
            None => class_from_status(context.status),
        },
    };
    let retry_after = parse_retry_after_header(context.headers)?;
    Ok(ServiceError {
        class,
        code,
        retry_after,
    })
}

fn body_code(context: super::ResponseContext<'_>) -> Result<Option<String>, super::DecodeError> {
    let content_type = context
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().map_or(value, str::trim));
    if matches!(
        content_type,
        Some("application/json" | "application/x-amz-json-1.0" | "application/x-amz-json-1.1")
    ) {
        return json::code(context.body, context.limits);
    }
    match context.protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 | Protocol::RestJson1 => {
            json::code(context.body, context.limits)
        }
        Protocol::AwsQuery | Protocol::Ec2Query | Protocol::RestXml => {
            xml::code(context.body, context.limits)
        }
        Protocol::RpcV2Cbor => cbor::code(context.body, context.limits),
    }
}

fn error_header(headers: &http::HeaderMap) -> Result<Option<String>, super::DecodeError> {
    for name in [
        "x-amzn-errortype",
        "x-amzn-query-error",
        "smithy-error-code",
        "x-amz-error-code",
    ] {
        if let Some(value) = headers.get(name) {
            let value = value
                .to_str()
                .map_err(|_| super::DecodeError::Invalid("invalid service error header"))?;
            return normalize_code(value).map(Some);
        }
    }
    Ok(None)
}

fn modeled_class(
    context: super::ResponseContext<'_>,
    code: Option<&str>,
) -> Result<Option<ErrorClass>, super::DecodeError> {
    let Some(code) = code else {
        return Ok(None);
    };
    find_modeled_class(context, code)
}

fn find_modeled_class(
    context: super::ResponseContext<'_>,
    code: &str,
) -> Result<Option<ErrorClass>, super::DecodeError> {
    for target in context.operation.errors()? {
        if let Some(class) = target_class(context.service, target?, code)? {
            return Ok(Some(class));
        }
    }
    Ok(None)
}

fn target_class(
    service: inventory_core::catalog::Service<'_>,
    target: inventory_core::catalog::Target,
    code: &str,
) -> Result<Option<ErrorClass>, super::DecodeError> {
    let inventory_core::catalog::Target::Shape(index) = target else {
        return Ok(None);
    };
    let shape = service
        .shape(index)?
        .ok_or(super::DecodeError::Invalid("missing modeled error shape"))?;
    if !error_matches(shape, code)? {
        return Ok(None);
    }
    Ok(shape.error()?.map(|error| class_from_modeled(error, code)))
}

fn error_matches(shape: Shape<'_>, code: &str) -> Result<bool, super::DecodeError> {
    let name = shape.name()?;
    let normalized_name = name.split_once('#').map_or(name, |(_, value)| value);
    let metadata = shape.error()?;
    Ok(code == normalized_name
        || metadata
            .and_then(|error| error.code)
            .is_some_and(|alias| code == alias))
}

fn class_from_modeled(error: ModeledError<'_>, code: &str) -> ErrorClass {
    if error.throttling || error.status == Some(429) {
        ErrorClass::Throttle
    } else if let Some(class) = class_from_code(Some(code)) {
        class
    } else if error
        .status
        .is_some_and(|status| status == 401 || status == 403)
    {
        ErrorClass::AccessDenied
    } else if error.status.is_some_and(|status| status >= 500)
        || error.retryable
        || error.kind == ModeledKind::Server
    {
        ErrorClass::Retryable
    } else {
        ErrorClass::Fatal
    }
}

fn normalize_code(value: &str) -> Result<String, super::DecodeError> {
    let trimmed = value.trim();
    let namespace = trimmed
        .split_once('#')
        .map_or(trimmed, |(_, suffix)| suffix);
    let before_semicolon = namespace
        .split_once(';')
        .map_or(namespace, |(name, _)| name);
    let code = before_semicolon
        .split_once(':')
        .map_or(before_semicolon, |(name, _)| name);
    if code.is_empty() || code.len() > 256 || code.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(super::DecodeError::Invalid("invalid service error code"));
    }
    let mut owned = String::new();
    owned.try_reserve_exact(code.len())?;
    owned.push_str(code);
    Ok(owned)
}

fn class_from_code(code: Option<&str>) -> Option<ErrorClass> {
    match code? {
        "AccessDenied"
        | "AccessDeniedException"
        | "Unauthorized"
        | "UnauthorizedException"
        | "UnrecognizedClientException"
        | "InvalidClientTokenId" => Some(ErrorClass::AccessDenied),
        "NotImplemented"
        | "NotSupported"
        | "UnsupportedOperation"
        | "UnsupportedOperationException"
        | "UnknownOperationException"
        | "UnknownOperationError"
        | "UnknownService"
        | "InvalidAction" => Some(ErrorClass::NotSupported),
        "ExpiredToken" | "ExpiredTokenException" | "TokenExpired" => Some(ErrorClass::ExpiredToken),
        "ExpiredSignature" => Some(ErrorClass::ExpiredSignature),
        "Throttling"
        | "ThrottlingException"
        | "TooManyRequestsException"
        | "RequestLimitExceeded" => Some(ErrorClass::Throttle),
        _ => None,
    }
}

/// Upper bound on a server retry hint; larger hints are clamped, not dropped.
const MAX_RETRY_AFTER_SECONDS: u64 = 300;

/// Decode a delta-seconds `retry-after` header, clamped to `MAX_RETRY_AFTER_SECONDS`.
/// Non-numeric values (including HTTP-date) carry no usable hint and yield `None`.
fn parse_retry_after_header(
    headers: &http::HeaderMap,
) -> Result<Option<std::time::Duration>, super::DecodeError> {
    let Some(value) = headers.get("retry-after") else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .map_err(|_| super::DecodeError::Invalid("invalid retry-after header value"))?
        .trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(None);
    }
    // All-digit text only fails to parse on u64 overflow, which exceeds the cap.
    let seconds = text
        .parse::<u64>()
        .unwrap_or(MAX_RETRY_AFTER_SECONDS)
        .min(MAX_RETRY_AFTER_SECONDS);
    Ok(Some(std::time::Duration::from_secs(seconds)))
}

fn class_from_status(status: StatusCode) -> ErrorClass {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        ErrorClass::AccessDenied
    } else if status == StatusCode::TOO_MANY_REQUESTS {
        ErrorClass::Throttle
    } else if status == StatusCode::NOT_IMPLEMENTED {
        ErrorClass::NotSupported
    } else if status.is_server_error() {
        ErrorClass::Retryable
    } else {
        ErrorClass::Fatal
    }
}
