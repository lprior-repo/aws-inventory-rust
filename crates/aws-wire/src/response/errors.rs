use crate::request::Protocol;
use http::StatusCode;
use inventory_core::catalog::{ErrorKind as ModeledKind, ModeledError, Shape};

mod cbor;
mod field;
mod json;
mod xml;

use field::{Field, MAX_FIELD_BYTES, error_header};

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
        Some(code) => Some(own_text(code)?),
        None => body_field(context, Field::Code)?,
    };
    let class = match modeled_class(context, code.as_deref())? {
        Some(class) => class,
        None => match class_from_code(code.as_deref()) {
            Some(class) => class,
            None => class_from_status(context.status),
        },
    };
    let retry_after = parse_retry_after_header(context.headers);
    Ok(ServiceError {
        class,
        code,
        retry_after,
    })
}

pub fn message(context: super::ResponseContext<'_>) -> Result<Option<String>, super::DecodeError> {
    if context.body.len() > context.limits.body_bytes() {
        return Err(super::DecodeError::Limit("response body"));
    }
    if let Some(header) = error_header(context.headers)?
        && body_field(context, Field::Code)?.is_some_and(|body| body != header)
    {
        return Err(Field::Code.invalid());
    }
    body_field(context, Field::Message)
}

pub fn header_body_codes_disagree(context: super::ResponseContext<'_>) -> bool {
    if context.body.is_empty() || context.body.len() > context.limits.body_bytes() {
        return false;
    }
    let Ok(Some(header)) = error_header(context.headers) else {
        return false;
    };
    matches!(body_field(context, Field::Code), Ok(Some(body)) if body != header)
}

fn body_field(
    context: super::ResponseContext<'_>,
    field: Field,
) -> Result<Option<String>, super::DecodeError> {
    let content_type = context
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().map_or(value, str::trim));
    if matches!(
        content_type,
        Some("application/json" | "application/x-amz-json-1.0" | "application/x-amz-json-1.1")
    ) {
        return json::scan(context.body, context.limits, field);
    }
    match context.protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 | Protocol::RestJson1 => {
            json::scan(context.body, context.limits, field)
        }
        Protocol::AwsQuery | Protocol::Ec2Query | Protocol::RestXml => {
            xml::scan(context.body, context.limits, field)
        }
        Protocol::RpcV2Cbor => {
            if context.body.trim_ascii_start().starts_with(b"{") {
                json::scan(context.body, context.limits, field)
            } else {
                cbor::scan(context.body, context.limits, field)
            }
        }
    }
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

fn normalized_code(value: &str) -> Result<&str, super::DecodeError> {
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
    if code.is_empty()
        || code.len() > MAX_FIELD_BYTES
        || code.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(Field::Code.invalid());
    }
    Ok(code)
}

fn normalize_code(value: &str) -> Result<String, super::DecodeError> {
    own_text(normalized_code(value)?)
}

fn own_text(value: &str) -> Result<String, super::DecodeError> {
    let mut owned = String::new();
    owned.try_reserve_exact(value.len())?;
    owned.push_str(value);
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
        | "RequestLimitExceeded"
        | "SlowDown"
        | "PriorRequestNotComplete" => Some(ErrorClass::Throttle),
        "AmbiguousService" => Some(ErrorClass::Fatal),
        _ => None,
    }
}

const MAX_RETRY_AFTER_SECONDS: u64 = 300;

fn parse_retry_after_header(headers: &http::HeaderMap) -> Option<std::time::Duration> {
    let text = headers
        .get(http::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let seconds = text
        .parse::<u64>()
        .map_or(MAX_RETRY_AFTER_SECONDS, |value| {
            value.min(MAX_RETRY_AFTER_SECONDS)
        });
    Some(std::time::Duration::from_secs(seconds))
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
