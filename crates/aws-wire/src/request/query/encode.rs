use super::{form, serialize};
use crate::request::{
    EncodeError, Protocol, RequestContext,
    common::{BoundedBody, ensure_service_protocol, finish},
};
use http::{
    HeaderMap,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};

pub fn encode(context: RequestContext<'_>) -> Result<http::Request<bytes::Bytes>, EncodeError> {
    let protocol_name = protocol_name(context.protocol)?;
    ensure_service_protocol(context, protocol_name)?;
    let action = context.operation.name()?;
    let version = context.service.version()?;
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut first = true;
    form::pair(&mut body, &mut first, "Action", action)?;
    form::pair(&mut body, &mut first, "Version", version)?;
    serialize::input(context, &mut body, &mut first)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let body = body.into_vec();
    let length = body.len().to_string();
    headers.insert(CONTENT_LENGTH, http::HeaderValue::from_str(&length)?);
    let endpoint = crate::request::endpoint::resolve(context)?;
    let adjusted = RequestContext {
        endpoint: &endpoint,
        ..context
    };
    finish(adjusted, "POST", Some("/"), headers, body)
}

fn protocol_name(protocol: Protocol) -> Result<&'static str, EncodeError> {
    match protocol {
        Protocol::AwsQuery => Ok("aws.protocols#awsQuery"),
        Protocol::Ec2Query => Ok("aws.protocols#ec2Query"),
        _ => Err(EncodeError::Unsupported(
            "protocol is not AWS Query or EC2 Query",
        )),
    }
}
