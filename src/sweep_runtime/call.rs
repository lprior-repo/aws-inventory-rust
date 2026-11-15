use std::time::SystemTime;

use aws_wire::{
    request::Protocol,
    sign::{Body, PathMode, PayloadHeader, Region, Request, Scope, SigningName},
    transport::{Client, Response},
};
use inventory_aws::model_credentials::{CredentialError, ProfileCredentials};
use inventory_core::catalog::runtime::{CallablePlan, ResolvedEndpoint, RuntimeProtocol};
use inventory_core::record::Record;
use inventory_engine::limiter::{ApiError, ApiExecutor};

use super::{SweepError, input::InputField, request};

pub(super) struct Call<'a, 'catalog> {
    pub client: &'a Client,
    pub executor: &'a ApiExecutor,
    pub credentials: &'a ProfileCredentials,
    pub plan: &'a CallablePlan<'catalog>,
    pub endpoint: &'a ResolvedEndpoint,
    pub url: &'a str,
    pub base: &'a Record,
    pub fields: &'a [InputField],
}

pub(super) struct DecodedResponse {
    pub record: Record,
    pub headers: http::HeaderMap,
}

impl Call<'_, '_> {
    pub(super) async fn run(&self) -> Result<DecodedResponse, SweepError> {
        let bucket = self.plan.service().name().map_err(SweepError::Model)?;
        let response = self
            .executor
            .execute(bucket, || async { self.send_attempt().await })
            .await?;
        decode_response(self.plan, response)
    }

    async fn send_attempt(&self) -> Result<Response, ApiError> {
        let request =
            request::encode(self.plan, self.url, self.base, self.fields).map_err(permanent)?;
        let credentials = self.credentials.resolve().await.map_err(credential_error)?;
        let scope = Scope {
            region: Region::new(&self.endpoint.signing_region).map_err(permanent)?,
            name: SigningName::new(&self.endpoint.signing_name).map_err(permanent)?,
        };
        sign_and_send(self, request, &credentials, &scope)
            .await
            .and_then(|response| validate_status(self.plan, response))
    }
}

async fn sign_and_send(
    call: &Call<'_, '_>,
    request: http::Request<bytes::Bytes>,
    credentials: &aws_wire::sign::Credentials,
    scope: &Scope<'_>,
) -> Result<Response, ApiError> {
    let (mut parts, body) = request.into_parts();
    let uri = parts.uri.to_string();
    let path_mode = path_mode(call.endpoint.path_mode);
    let payload_header = match call.endpoint.path_mode {
        inventory_core::catalog::runtime::EndpointPathMode::S3 => PayloadHeader::Include,
        inventory_core::catalog::runtime::EndpointPathMode::Standard => PayloadHeader::Omit,
    };
    let signature = aws_wire::sign::sign(
        credentials,
        scope,
        SystemTime::now(),
        Request {
            method: &parts.method,
            uri: &uri,
            headers: &parts.headers,
            body: Body::Bytes(&body),
            payload_header,
        },
        path_mode,
    )
    .map_err(permanent)?;
    parts.headers.extend(signature);
    call.client
        .send(http::Request::from_parts(parts, body.clone()))
        .await
        .map_err(transport)
}

fn decode_response(
    plan: &CallablePlan<'_>,
    response: Response,
) -> Result<DecodedResponse, SweepError> {
    let limits = aws_wire::response::DecodeLimits::default();
    let limits = if matches!(plan.purpose(),inventory_core::catalog::runtime::PlanPurpose::TagEndpoint(_)) {
        limits.preserve_field_presence()
    } else { limits };
    let context = aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: protocol(plan.protocol()),
        body: &response.body,
        headers: &response.headers,
        status: response.status,
        limits,
    };
    let record = aws_wire::response::decode(context).map_err(SweepError::from)?;
    Ok(DecodedResponse {
        record,
        headers: response.headers,
    })
}

fn validate_status(plan: &CallablePlan<'_>, response: Response) -> Result<Response, ApiError> {
    if response.status.is_success() {
        return Ok(response);
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
        Ok(error) => Err(classify_service(error)),
        Err(_) => Err(status_fallback(response.status)),
    }
}

fn classify_service(error: aws_wire::response::errors::ServiceError) -> ApiError {
    let message = error.to_string();
    match error.class {
        aws_wire::response::errors::ErrorClass::Throttle => ApiError::Throttled(message),
        aws_wire::response::errors::ErrorClass::Retryable
        | aws_wire::response::errors::ErrorClass::ExpiredToken
        | aws_wire::response::errors::ErrorClass::ExpiredSignature => ApiError::Transient(message),
        aws_wire::response::errors::ErrorClass::AccessDenied => ApiError::Denied(message),
        aws_wire::response::errors::ErrorClass::NotSupported => ApiError::UnsupportedType(message),
        aws_wire::response::errors::ErrorClass::Fatal => ApiError::Permanent(message),
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

fn credential_error(error: CredentialError) -> ApiError {
    let message = error.to_string();
    match error {
        CredentialError::Provider(_) => ApiError::Transient(message),
        _ => ApiError::Permanent(message),
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
mod tests {
    use super::*;

    #[test]
    fn denied_http_status_is_not_retried() {
        assert!(matches!(
            status_fallback(http::StatusCode::FORBIDDEN),
            ApiError::Denied(_)
        ));
        assert!(matches!(
            status_fallback(http::StatusCode::TOO_MANY_REQUESTS),
            ApiError::Throttled(_)
        ));
    }
}
