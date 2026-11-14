use super::{field_value, set_content_type, EncodeError, Protocol, RequestContext};
use super::super::common::BoundedBody;
use http::header::{HeaderMap, HeaderValue};
use inventory_core::catalog::{HttpBinding, Target};

mod json;
mod xml;

pub(super) fn encode(context: RequestContext<'_>, headers: &mut HeaderMap) -> Result<Vec<u8>, EncodeError> {
    let Target::Shape(index) = context.operation.input()? else { return Ok(Vec::new()) };
    let input = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
    if input.wire()?.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
    let payload = input.members()?.try_fold(None, |found, member| {
        let member = member?;
        if matches!(member.wire()?.binding()?, HttpBinding::Payload) { Ok(Some(member)) } else { Ok(found) }
    })?;
    if let Some(member) = payload {
        let value = match field_value(context, member)? {
            Some(inventory_core::record::ValueRef::Null) => return if member.required {
                Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required payload member is null" })
            } else {
                Ok(Vec::new())
            },
            Some(value) => value,
            None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required payload member is missing" }),
            None => return Ok(Vec::new()),
        };
        if member.wire()?.streaming() || payload_streaming(context, member.target)? {
            return Err(EncodeError::Unsupported("streaming request body"));
        }
        let media_type = member.wire()?.media_type()?.or(payload_media_type(context, member.target)?);
        content_type(headers, media_type.or(payload_default_media_type(context.protocol, member.target, context)?))?;
        return body_for_payload(context, member, value);
    }
    let mut has_document = false;
    for item in input.members()? {
        if matches!(item?.wire()?.binding()?, HttpBinding::Document) { has_document = true; }
    }
    if !has_document { return Ok(Vec::new()); }
    if context.protocol == Protocol::RestJson1 {
        set_content_type(headers, "application/json");
        json::encode(context, input)
    } else {
        set_content_type(headers, "application/xml");
        xml::encode(context, input)
    }
}

fn body_for_payload(context: RequestContext<'_>, member: inventory_core::catalog::Member<'_>, value: inventory_core::record::ValueRef<'_>) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    match value {
        inventory_core::record::ValueRef::Bytes(bytes) => body.write(bytes)?,
        inventory_core::record::ValueRef::Text(text) => body.write(text.as_bytes())?,
        value if context.protocol == Protocol::RestJson1 => return json::encode_payload(context, member.target, value),
        value if context.protocol == Protocol::RestXml => return xml::encode_payload(context, member.target, value),
        _ => return Err(EncodeError::Unsupported("structured payload binding")),
    }
    Ok(body.into_vec())
}


fn payload_media_type<'a>(context: RequestContext<'a>, target: Target) -> Result<Option<&'a str>, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.wire()?.media_type()?),
        Target::Prelude(_) => Ok(None),
    }
}

fn payload_default_media_type(protocol: Protocol, target: Target, context: RequestContext<'_>) -> Result<Option<&'static str>, EncodeError> {
    let kind = match target {
        Target::Shape(index) => context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.kind()?,
        Target::Prelude(kind) => kind,
    };
    Ok(Some(match kind {
        "blob" | "Blob" => "application/octet-stream",
        "string" | "String" => "text/plain",
        _ if protocol == Protocol::RestJson1 => "application/json",
        _ => "application/xml",
    }))
}

fn payload_streaming(context: RequestContext<'_>, target: Target) -> Result<bool, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.wire()?.streaming()),
        Target::Prelude(_) => Ok(false),
    }
}
fn content_type(headers: &mut HeaderMap, media_type: Option<&str>) -> Result<(), EncodeError> {
    if let Some(media_type) = media_type {
        headers.insert(http::header::CONTENT_TYPE, HeaderValue::from_str(media_type)?);
    }
    Ok(())
}
