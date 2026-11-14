mod cbor;
pub(crate) mod json;
mod wire;
mod wire_shape;
mod wire_scalar;
mod wire_time;
mod wire_time_format;
mod wire_value;
mod wire_decimal;
mod wire_member;

use super::{EncodeError, Protocol, RequestContext, common::{BoundedBody, finish}};
use http::{HeaderMap, HeaderValue, header::{ACCEPT, CONTENT_TYPE, HeaderName}};
use inventory_core::catalog::Target;

pub fn encode(context: RequestContext<'_>) -> Result<http::Request<bytes::Bytes>, EncodeError> {
    let content_type = content_type(context.protocol)?;
    check_protocol(context)?;
    let input = context.operation.input()?;
    let has_input = validate_input(context, input)?;
    let body = encode_body(context, input, has_input)?;
    let headers = request_headers(context, has_input, content_type)?;
    let target = request_target(context)?;
    finish(context, "POST", target.as_deref(), headers, body)
}
fn request_target(context: RequestContext<'_>) -> Result<Option<String>, EncodeError> {
    match context.protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 => Ok(None),
        Protocol::RpcV2Cbor => rpc_target(context).map(Some),
        _ => Err(EncodeError::Unsupported("request body protocol")),
    }
}

fn rpc_target(context: RequestContext<'_>) -> Result<String, EncodeError> {
    let shape_id = context.service.shape_id()?;
    let (_, service_name) = shape_id.split_once('#').ok_or(EncodeError::InvalidValue {
        field: None,
        reason: "service shape ID has no namespace separator",
    })?;
    let operation_name = context.operation.name()?;
    let capacity = "/service/".len()
        .checked_add(service_name.len())
        .and_then(|size| size.checked_add("/operation/".len()))
        .and_then(|size| size.checked_add(operation_name.len()))
        .ok_or(EncodeError::Limit("URI"))?;
    if capacity > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    let mut target = String::new();
    target.try_reserve_exact(capacity)?;
    target.push_str("/service/");
    target.push_str(service_name);
    target.push_str("/operation/");
    target.push_str(operation_name);
    Ok(target)
}

fn content_type(protocol: Protocol) -> Result<&'static str, EncodeError> {
    match protocol {
        Protocol::AwsJson10 => Ok("application/x-amz-json-1.0"),
        Protocol::AwsJson11 => Ok("application/x-amz-json-1.1"),
        Protocol::RpcV2Cbor => Ok("application/cbor"),
        _ => Err(EncodeError::Unsupported("request body protocol")),
    }
}

fn validate_input(context: RequestContext<'_>, input: Target) -> Result<bool, EncodeError> {
    let has_input = input != Target::Prelude("unit");
    if !has_input {
        for entry in context.input.iter()? {
            entry?;
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "unit operation input contains members",
            });
        }
    }
    if let Target::Shape(index) = input {
        let shape = context.service.shape(index)?.ok_or(EncodeError::InvalidValue {
            field: None,
            reason: "operation input shape is missing",
        })?;
        if shape.wire()?.streaming() {
            return Err(EncodeError::Unsupported("streaming input shape"));
        }
    }
    Ok(has_input)
}

fn encode_body(
    context: RequestContext<'_>,
    input: Target,
    has_input: bool,
) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    if has_input {
        wire::encode(
            context.service,
            context.protocol,
            input,
            context.input,
            context.idempotency_token,
            &mut body,
        )?;
    }
    Ok(body.into_vec())
}

fn request_headers(
    context: RequestContext<'_>,
    has_input: bool,
    content_type: &'static str,
) -> Result<HeaderMap, EncodeError> {
    match context.protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 => json_headers(context, has_input, content_type),
        Protocol::RpcV2Cbor => cbor_headers(context, has_input, content_type),
        _ => Err(EncodeError::Unsupported("request body protocol")),
    }
}

fn json_headers(
    context: RequestContext<'_>,
    has_input: bool,
    content_type: &'static str,
) -> Result<HeaderMap, EncodeError> {
    let mut headers = HeaderMap::new();
    if has_input {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    }
    headers.insert(
        HeaderName::from_static("x-amz-target"),
        json_target_header(context)?,
    );
    Ok(headers)
}

fn json_target_header(context: RequestContext<'_>) -> Result<HeaderValue, EncodeError> {
    let shape_id = context.service.shape_id()?;
    let (_, prefix) = shape_id.split_once('#').ok_or(EncodeError::InvalidValue {
        field: None,
        reason: "service shape ID has no namespace separator",
    })?;
    let operation_name = context.operation.name()?;
    let capacity = prefix.len().checked_add(operation_name.len())
        .and_then(|size| size.checked_add(1))
        .ok_or(EncodeError::Limit("X-Amz-Target"))?;
    let header_bytes = "x-amz-target".len().checked_add(4)
        .and_then(|size| size.checked_add(capacity))
        .ok_or(EncodeError::Limit("headers"))?;
    if header_bytes > context.limits.header_bytes() {
        return Err(EncodeError::Limit("headers"));
    }
    let mut target = String::new();
    target.try_reserve_exact(capacity)?;
    target.push_str(prefix);
    target.push('.');
    target.push_str(operation_name);
    Ok(HeaderValue::try_from(target)?)
}

fn cbor_headers(
    context: RequestContext<'_>,
    has_input: bool,
    content_type: &'static str,
) -> Result<HeaderMap, EncodeError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/cbor"));
    headers.insert(
        HeaderName::from_static("smithy-protocol"),
        HeaderValue::from_static("rpc-v2-cbor"),
    );
    if context.service.aws_query_compatible()? {
        headers.insert(
            HeaderName::from_static("x-amzn-query-mode"),
            HeaderValue::from_static("true"),
        );
    }
    if has_input {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    }
    Ok(headers)
}

fn check_protocol(context: RequestContext<'_>) -> Result<(), EncodeError> {
    let expected = match context.protocol {
        Protocol::AwsJson10 => "aws.protocols#awsJson1_0",
        Protocol::AwsJson11 => "aws.protocols#awsJson1_1",
        Protocol::RpcV2Cbor => "smithy.protocols#rpcv2Cbor",
        _ => return Err(EncodeError::Unsupported("request body protocol")),
    };
    let protocols = context.service.protocols()?;
    for protocol in protocols {
        if protocol? == expected {
            return Ok(());
        }
    }
    Err(EncodeError::Unsupported("service protocol does not match request protocol"))
}
