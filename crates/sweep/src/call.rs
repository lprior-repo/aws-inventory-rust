use std::time::SystemTime;

use crate::{CredentialFailure, CredentialSource};
use aws_wire::{
    request::Protocol,
    sign::{Body, PathMode, PayloadHeader, Region, Request, Scope, SigningName},
    transport::{Client, Response},
};
use inventory_core::catalog::runtime::{
    CallablePlan, PlanPurpose, ResolvedEndpoint, RuntimeProtocol,
};
use inventory_core::record::Record;
use inventory_engine::limiter::{ApiError, ApiExecutor, RetryKind};

use super::execution::page::cursor;
use super::{SweepError, input::InputField, request};

mod absence;
use absence::ValidatedResponse;

pub(super) struct Call<'a, 'catalog, C: CredentialSource> {
    pub client: &'a Client,
    pub executor: &'a ApiExecutor,
    pub credentials: &'a C,
    pub plan: &'a CallablePlan<'catalog>,
    pub endpoint: &'a ResolvedEndpoint,
    pub url: &'a str,
    pub base: &'a Record,
    pub fields: &'a [InputField],
    pub metrics: &'a crate::metrics::Accumulators,
}

pub(super) struct DecodedResponse {
    pub record: Record,
    pub headers: http::HeaderMap,
    pub http_version: http::Version,
}

impl<C: CredentialSource> Call<'_, '_, C> {
    pub(super) async fn run(&self) -> Result<DecodedResponse, SweepError> {
        let bucket = self.plan.service().name().map_err(SweepError::Model)?;
        let response = self
            .executor
            .execute(bucket, || async { self.send_attempt().await })
            .await?;
        let started = std::time::Instant::now();
        let response = response.decode(self.plan);
        self.metrics.decoded(started);
        response
    }

    async fn send_attempt(&self) -> Result<ValidatedResponse, ApiError> {
        let request =
            request::encode(self.plan, self.url, self.base, self.fields).map_err(permanent)?;
        let credentials = self.credentials.resolve().await.map_err(credential_error)?;
        let scope = Scope {
            region: Region::new(&self.endpoint.signing_region).map_err(permanent)?,
            name: SigningName::new(&self.endpoint.signing_name).map_err(permanent)?,
        };
        let started = std::time::Instant::now();
        let response = sign_and_send(self, request, &credentials, &scope).await;
        let status = response
            .as_ref()
            .ok()
            .map(|response| response.status.as_u16());
        let result = response.and_then(|response| validate_status(self.plan, response));
        tracing::debug!(
            service = self.plan.service_name(),
            operation = self.plan.operation_name(),
            elapsed_ms = started.elapsed().as_millis(),
            status,
            error = ?result.as_ref().err(),
            "sweep API attempt completed"
        );
        result
    }
}

async fn sign_and_send(
    call: &Call<'_, '_, impl CredentialSource>,
    request: http::Request<bytes::Bytes>,
    credentials: &aws_wire::sign::Credentials,
    scope: &Scope<'_>,
) -> Result<Response, ApiError> {
    let (mut parts, body) = request.into_parts();
    let signature = signature(call, &parts, &body, credentials, scope)?;
    parts.headers.extend(signature);
    call.client
        .send(http::Request::from_parts(parts, body))
        .await
        .map_err(transport)
}

fn signature(
    call: &Call<'_, '_, impl CredentialSource>,
    parts: &http::request::Parts,
    body: &[u8],
    credentials: &aws_wire::sign::Credentials,
    scope: &Scope<'_>,
) -> Result<http::HeaderMap, ApiError> {
    aws_wire::sign::sign(
        credentials,
        scope,
        SystemTime::now(),
        Request {
            method: &parts.method,
            uri: &parts.uri.to_string(),
            headers: &parts.headers,
            body: Body::Bytes(body),
            payload_header: payload_header(call.endpoint.path_mode),
        },
        path_mode(call.endpoint.path_mode),
    )
    .map_err(permanent)
}

fn payload_header(mode: inventory_core::catalog::runtime::EndpointPathMode) -> PayloadHeader {
    match mode {
        inventory_core::catalog::runtime::EndpointPathMode::S3 => PayloadHeader::Include,
        inventory_core::catalog::runtime::EndpointPathMode::Standard => PayloadHeader::Omit,
    }
}

fn decode_response(
    plan: &CallablePlan<'_>,
    response: Response,
) -> Result<DecodedResponse, SweepError> {
    let tag_endpoint = matches!(plan.purpose(), PlanPurpose::TagEndpoint(_));
    let record = decode_record(plan, &response, tag_endpoint)?;
    if !tag_endpoint {
        confirm_truncation_presence(plan, &response, &record)?;
    }
    Ok(DecodedResponse {
        record,
        headers: response.headers,
        http_version: response.version,
    })
}

fn decode_record(
    plan: &CallablePlan<'_>,
    response: &Response,
    preserve_presence: bool,
) -> Result<Record, SweepError> {
    let limits =
        aws_wire::response::DecodeLimits::default().excluding_members(plan.excluded_members());
    let limits = if preserve_presence {
        limits.preserve_field_presence()
    } else {
        limits
    };
    let context = aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: protocol(plan.protocol()),
        body: &response.body,
        headers: &response.headers,
        status: response.status,
        limits,
    };
    aws_wire::response::decode(context).map_err(SweepError::from)
}

/// Enumeration decodes apply SDK defaults, which turn an omitted required or defaulted
/// truncation flag into `false`; the cursor reads `false` as completion. A `false` flag is
/// therefore accepted only when the presence-preserving decode also carries it, so an omitted
/// flag fails here instead of ending enumeration.
fn confirm_truncation_presence(
    plan: &CallablePlan<'_>,
    response: &Response,
    record: &Record,
) -> Result<(), SweepError> {
    let pagination = plan.pagination();
    if cursor::truncation_flag(pagination, record)? != Some(false) {
        return Ok(());
    }
    let evidence = decode_record(plan, response, true)?;
    cursor::truncation_flag(pagination, &evidence).map(|_| ())
}

fn validate_status(
    plan: &CallablePlan<'_>,
    response: Response,
) -> Result<ValidatedResponse, ApiError> {
    if response.status.is_success() {
        return Ok(ValidatedResponse::Page(response));
    }
    let context = aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: protocol(plan.protocol()),
        body: &response.body,
        headers: &response.headers,
        status: response.status,
        limits: aws_wire::response::DecodeLimits::default(),
    };
    match aws_wire::response::errors::decode(context) {
        Ok(error) => absence::classify(plan, error),
        Err(_) => Err(status_fallback(response.status)),
    }
}

fn classify_service(error: aws_wire::response::errors::ServiceError) -> ApiError {
    let message = error.to_string();
    let class = error.class;
    match class {
        aws_wire::response::errors::ErrorClass::Throttle => match error.retry_after {
            Some(duration) => retry_after(message, duration, RetryKind::Throttled),
            None => ApiError::Throttled(message),
        },
        aws_wire::response::errors::ErrorClass::Retryable
        | aws_wire::response::errors::ErrorClass::ExpiredToken
        | aws_wire::response::errors::ErrorClass::ExpiredSignature => match error.retry_after {
            Some(duration) => retry_after(message, duration, RetryKind::Transient),
            None => ApiError::Transient(message),
        },
        aws_wire::response::errors::ErrorClass::AccessDenied => ApiError::Denied(message),
        aws_wire::response::errors::ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        aws_wire::response::errors::ErrorClass::Fatal => ApiError::Permanent(message),
    }
}

fn retry_after(message: String, duration: std::time::Duration, kind: RetryKind) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message,
        retry_after: inventory_engine::limiter::RetryAfter::Delay(duration),
    }
}

fn status_fallback(status: http::StatusCode) -> ApiError {
    match status.as_u16() {
        429 => ApiError::Throttled(format!("HTTP {status}")),
        401 | 403 => ApiError::Denied(format!("HTTP {status}")),
        500..=599 => ApiError::Transient(format!("HTTP {status}")),
        _ => ApiError::Permanent(format!("HTTP {status}")),
    }
}

fn credential_error(error: CredentialFailure) -> ApiError {
    if error.retryable {
        ApiError::Transient(error.message)
    } else {
        ApiError::Permanent(error.message)
    }
}

fn transport(error: aws_wire::transport::TransportError) -> ApiError {
    let message = error.to_string();
    match error {
        aws_wire::transport::TransportError::Timeout => ApiError::Timeout,
        aws_wire::transport::TransportError::Connect(_)
        | aws_wire::transport::TransportError::Io(_)
        | aws_wire::transport::TransportError::Protocol(_) => ApiError::Transient(message),
        _ => ApiError::Permanent(message),
    }
}

fn permanent(error: impl std::fmt::Display) -> ApiError {
    ApiError::Permanent(error.to_string())
}

fn protocol(protocol: RuntimeProtocol) -> Protocol {
    match protocol {
        RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        RuntimeProtocol::RpcV2Cbor => Protocol::RpcV2Cbor,
        RuntimeProtocol::AwsQuery => Protocol::AwsQuery,
        RuntimeProtocol::Ec2Query => Protocol::Ec2Query,
        RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        RuntimeProtocol::RestXml => Protocol::RestXml,
    }
}

fn path_mode(mode: inventory_core::catalog::runtime::EndpointPathMode) -> PathMode {
    match mode {
        inventory_core::catalog::runtime::EndpointPathMode::Standard => PathMode::Standard,
        inventory_core::catalog::runtime::EndpointPathMode::S3 => PathMode::S3,
    }
}

#[cfg(test)]
mod tests;
