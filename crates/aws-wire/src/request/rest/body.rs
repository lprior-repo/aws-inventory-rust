use super::super::common::BoundedBody;
use super::{EncodeError, Protocol, RequestContext, field_value, set_content_type};
use http::header::{HeaderMap, HeaderValue};
use inventory_core::catalog::{HttpBinding, Target};

mod json;
mod xml;

pub(super) fn encode(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
) -> Result<Vec<u8>, EncodeError> {
    let Target::Shape(index) = context.operation.input()? else {
        return Ok(Vec::new());
    };
    let input = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    if input.wire()?.streaming()? {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let payload = input.members()?.try_fold(None, |found, member| {
        let member = member?;
        if matches!(member.wire()?.binding()?, HttpBinding::Payload) {
            Ok::<_, EncodeError>(Some(member))
        } else {
            Ok(found)
        }
    })?;
    if let Some(member) = payload {
        let value = match field_value(context, member)? {
            Some(inventory_core::record::ValueRef::Null) => {
                return if member.required {
                    Err(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "required payload member is null",
                    })
                } else {
                    absent_payload(context, member.target, headers)
                };
            }
            Some(value) => value,
            None if member.required => {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required payload member is missing",
                });
            }
            None => return absent_payload(context, member.target, headers),
        };
        let streaming = member.wire()?.streaming()? || payload_streaming(context, member.target)?;
        if streaming && payload_kind(context, member.target)? != "blob" {
            return Err(EncodeError::Unsupported("streaming request body"));
        }
        let media_type = member
            .wire()?
            .media_type()?
            .or(payload_media_type(context, member.target)?);
        content_type(
            headers,
            media_type.or(payload_default_media_type(
                context.protocol,
                member.target,
                context,
            )?),
        )?;
        return body_for_payload(context, member, value);
    }
    let mut has_document = false;
    for item in input.members()? {
        if matches!(item?.wire()?.binding()?, HttpBinding::Document) {
            has_document = true;
        }
    }
    if !has_document {
        return Ok(Vec::new());
    }
    if context.protocol == Protocol::RestJson1 {
        set_content_type(headers, "application/json");
        json::encode(context, input)
    } else {
        set_content_type(headers, "application/xml");
        xml::encode(context, input)
    }
}

fn body_for_payload<'a>(
    context: RequestContext<'a>,
    member: inventory_core::catalog::Member<'a>,
    value: inventory_core::record::ValueRef<'a>,
) -> Result<Vec<u8>, EncodeError> {
    if context.protocol == Protocol::RestJson1 && payload_is_document(context, member.target)? {
        return json::encode_payload(context, member.target, value);
    }
    let mut body = BoundedBody::new(context.limits.body_bytes());
    match value {
        inventory_core::record::ValueRef::Bytes(bytes) => body.write(bytes)?,
        inventory_core::record::ValueRef::Text(text) => body.write(text.as_bytes())?,
        value if context.protocol == Protocol::RestJson1 => {
            return json::encode_payload(context, member.target, value);
        }
        value if context.protocol == Protocol::RestXml => {
            return xml::encode_payload(context, member.target, value, member.wire()?.xml_name()?);
        }
        _ => return Err(EncodeError::Unsupported("structured payload binding")),
    }
    Ok(body.into_vec())
}
fn payload_is_document(context: RequestContext<'_>, target: Target) -> Result<bool, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .kind()?
            == "document"),
        Target::Prelude(_) => Ok(false),
        _ => Err(EncodeError::Unsupported("unknown payload target")),
    }
}

fn payload_media_type<'a>(
    context: RequestContext<'a>,
    target: Target,
) -> Result<Option<&'a str>, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .wire()?
            .media_type()?),
        Target::Prelude(_) => Ok(None),
        _ => Err(EncodeError::Unsupported("unknown payload target")),
    }
}

fn absent_payload(
    context: RequestContext<'_>,
    target: Target,
    headers: &mut HeaderMap,
) -> Result<Vec<u8>, EncodeError> {
    if context.protocol != Protocol::RestJson1 || payload_kind(context, target)? != "structure" {
        return Ok(Vec::new());
    }
    content_type(headers, Some("application/json"))?;
    Ok(b"{}".to_vec())
}

fn payload_kind(context: RequestContext<'_>, target: Target) -> Result<&'static str, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .kind()?),
        Target::Prelude(kind) => Ok(kind),
        _ => Err(EncodeError::Unsupported("unknown payload target")),
    }
}

fn payload_default_media_type(
    protocol: Protocol,
    target: Target,
    context: RequestContext<'_>,
) -> Result<Option<&'static str>, EncodeError> {
    Ok(Some(match payload_kind(context, target)? {
        "blob" | "Blob" => "application/octet-stream",
        "string" | "String" | "enum" => "text/plain",
        _ if protocol == Protocol::RestJson1 => "application/json",
        _ => "application/xml",
    }))
}

fn payload_streaming(context: RequestContext<'_>, target: Target) -> Result<bool, EncodeError> {
    match target {
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .wire()?
            .streaming()?),
        Target::Prelude(_) => Ok(false),
        _ => Err(EncodeError::Unsupported("unknown streaming target")),
    }
}
fn content_type(headers: &mut HeaderMap, media_type: Option<&str>) -> Result<(), EncodeError> {
    if let Some(media_type) = media_type
        && !headers.contains_key(http::header::CONTENT_TYPE)
    {
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str(media_type)?,
        );
    }
    Ok(())
}
