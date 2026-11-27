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
    pub emulator_endpoint_override: bool,
    pub base: &'a Record,
    pub fields: &'a [InputField],
    pub metrics: &'a crate::metrics::Accumulators,
    pub account: &'a str,
    pub scope: &'a inventory_core::inventory::Region,
}

pub(super) struct DecodedResponse {
    pub record: Record,
    pub headers: http::HeaderMap,
    pub http_version: http::Version,
}

impl<C: CredentialSource> Call<'_, '_, C> {
    pub(super) async fn run(&self) -> Result<DecodedResponse, SweepError> {
        self.run_with_clock(SystemTime::now, || {
            request::encode(self.plan, self.url, self.base, self.fields)
        })
        .await
    }

    async fn run_with_clock(
        &self,
        now: impl Fn() -> SystemTime,
        encode: impl Fn() -> Result<http::Request<bytes::Bytes>, SweepError>,
    ) -> Result<DecodedResponse, SweepError> {
        let group = self.plan.service().name().map_err(SweepError::Model)?;
        let bucket = crate::bucket::key(self.account, self.scope, group);
        let request = encode();
        let response = self
            .executor
            .execute(&bucket, || async {
                let request = request.as_ref().map_err(permanent)?.clone();
                self.send_attempt(request, &now).await
            })
            .await?;
        let started = std::time::Instant::now();
        let response = response.decode(self.plan, self.emulator_endpoint_override);
        self.metrics.decoded(started);
        response
    }

    pub(super) async fn run_raw(&self) -> Result<DecodedResponse, SweepError> {
        let group = self.plan.service().name().map_err(SweepError::Model)?;
        let bucket = crate::bucket::key(self.account, self.scope, group);
        let request = request::encode(self.plan, self.url, self.base, self.fields);
        let response = self
            .executor
            .execute(&bucket, || async {
                let request = request.as_ref().map_err(permanent)?.clone();
                self.send_attempt(request, &SystemTime::now).await
            })
            .await?;
        let response = match response {
            ValidatedResponse::Page(response) => response,
            ValidatedResponse::Absent(reason) => return Err(SweepError::Absent(reason)),
        };
        if response.body.len() > 1_048_576 {
            return Err(SweepError::Operation(
                "comparison wire response exceeds one MiB",
            ));
        }
        let limits = if self.emulator_endpoint_override {
            aws_wire::response::DecodeLimits::default()
                .preserve_raw_values()
                .allow_emulator_numeric_timestamps()
        } else {
            aws_wire::response::DecodeLimits::default().preserve_raw_values()
        };
        let context = aws_wire::response::ResponseContext {
            service: self.plan.service(),
            operation: self.plan.operation(),
            protocol: protocol(self.plan.protocol()),
            body: &response.body,
            headers: &response.headers,
            status: response.status,
            limits,
        };
        Ok(DecodedResponse {
            record: aws_wire::response::decode(context)?,
            headers: response.headers,
            http_version: response.version,
        })
    }

    async fn send_attempt(
        &self,
        request: http::Request<bytes::Bytes>,
        now: &impl Fn() -> SystemTime,
    ) -> Result<ValidatedResponse, ApiError> {
        let credentials = self.credentials.resolve().await.map_err(credential_error)?;
        let scope = Scope {
            region: Region::new(&self.endpoint.signing_region).map_err(permanent)?,
            name: SigningName::new(&self.endpoint.signing_name).map_err(permanent)?,
        };
        if let Some(attempts) = &self.metrics.attempts {
            attempts
                .record(self.plan.service_name(), self.plan.operation_name())
                .map_err(permanent)?;
        }
        let started = std::time::Instant::now();
        let response = sign_and_send(self, request, &credentials, &scope, now()).await;
        let status = response
            .as_ref()
            .ok()
            .map(|response| response.status.as_u16());
        let result = response.and_then(|response| {
            validate_status(self.plan, response, self.account, self.scope.as_str())
        });
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
    now: SystemTime,
) -> Result<Response, ApiError> {
    let (mut parts, body) = request.into_parts();
    let signature = signature(call, &parts, &body, credentials, scope, now)?;
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
    now: SystemTime,
) -> Result<http::HeaderMap, ApiError> {
    aws_wire::sign::sign(
        credentials,
        scope,
        now,
        Request {
            method: &parts.method,
            uri: &parts.uri.to_string(),
            headers: &parts.headers,
            body: Body::Bytes(body),
            payload_header: payload_header(
                call.endpoint.path_mode,
                call.plan.service_name(),
                call.plan.operation_name(),
            ),
        },
        path_mode(call.endpoint.path_mode),
    )
    .map_err(permanent)
}

fn payload_header(
    mode: inventory_core::catalog::runtime::EndpointPathMode,
    service_name: &str,
    operation_name: &str,
) -> PayloadHeader {
    match (
        mode,
        service_name == "s3" && operation_name == "ListBuckets",
    ) {
        (inventory_core::catalog::runtime::EndpointPathMode::S3, _) | (_, true) => {
            PayloadHeader::Include
        }
        (inventory_core::catalog::runtime::EndpointPathMode::Standard, false) => PayloadHeader::Omit,
    }
}

fn decode_response(
    plan: &CallablePlan<'_>,
    response: Response,
    emulator_numeric_timestamps: bool,
) -> Result<DecodedResponse, SweepError> {
    let tag_endpoint = matches!(plan.purpose(), PlanPurpose::TagEndpoint(_));
    let record = decode_record(plan, &response, tag_endpoint, emulator_numeric_timestamps)?;
    if !tag_endpoint {
        confirm_truncation_presence(
            plan,
            &response,
            &record,
            emulator_numeric_timestamps,
        )?;
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
    emulator_numeric_timestamps: bool,
) -> Result<Record, SweepError> {
    let limits =
        aws_wire::response::DecodeLimits::default().excluding_members(plan.excluded_members());
    let limits = if emulator_numeric_timestamps {
        limits.allow_emulator_numeric_timestamps()
    } else {
        limits
    };
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

fn confirm_truncation_presence(
    plan: &CallablePlan<'_>,
    response: &Response,
    record: &Record,
    emulator_numeric_timestamps: bool,
) -> Result<(), SweepError> {
    let pagination = plan.pagination();
    if cursor::truncation_flag(pagination, record)? != Some(false) {
        return Ok(());
    }
    let evidence = decode_record(
        plan,
        response,
        true,
        emulator_numeric_timestamps,
    )?;
    cursor::truncation_flag(pagination, &evidence).map(|_| ())
}

fn validate_status(
    plan: &CallablePlan<'_>,
    response: Response,
    account: &str,
    region: &str,
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
        Ok(error) => absence::classify(plan, error, context, account, region),
        Err(error) => Err(status_fallback(response.status, &error)),
    }
}

fn classify_service(
    error: aws_wire::response::errors::ServiceError,
    status: http::StatusCode,
) -> ApiError {
    use aws_wire::response::errors::ErrorClass;

    let message = if error.code.is_some() {
        error.to_string()
    } else {
        format!("HTTP {status}")
    };
    let class = match status {
        http::StatusCode::TOO_MANY_REQUESTS => ErrorClass::Throttle,
        http::StatusCode::SERVICE_UNAVAILABLE => ErrorClass::Retryable,
        _ => error.class,
    };
    match class {
        ErrorClass::Throttle => match error.retry_after {
            Some(duration) => retry_after(message, duration, RetryKind::Throttled),
            None => ApiError::Throttled(message),
        },
        ErrorClass::Retryable | ErrorClass::ExpiredToken | ErrorClass::ExpiredSignature => {
            match error.retry_after {
                Some(duration) => retry_after(message, duration, RetryKind::Transient),
                None => ApiError::Transient(message),
            }
        }
        ErrorClass::AccessDenied => ApiError::Denied(message),
        ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        ErrorClass::Fatal => ApiError::Permanent(message),
    }
}

fn retry_after(message: String, duration: std::time::Duration, kind: RetryKind) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message,
        retry_after: inventory_engine::limiter::RetryAfter::Delay(duration),
    }
}

fn status_fallback(status: http::StatusCode, error: &aws_wire::response::DecodeError) -> ApiError {
    let message = format!("HTTP {status}: {error}");
    match status.as_u16() {
        429 => ApiError::Throttled(message),
        401 | 403 => ApiError::Denied(message),
        500..=599 => ApiError::Transient(message),
        _ => ApiError::Permanent(message),
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

#[cfg(test)]
mod retry_tests;
