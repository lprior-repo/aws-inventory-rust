use super::super::{timestamp, uri, write_header};
use crate::request::{EncodeError, RequestContext, body::json::base64, common::BoundedBody};
use http::header::HeaderMap;
use inventory_core::{
    catalog::{Target, TimestampFormat},
    record::{FieldId, KeyRef, ListRef, ValueRef},
};

pub(super) fn value(
    context: RequestContext<'_>,
    value: ValueRef<'_>,
    target: Target,
    field: FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<String, EncodeError> {
    if let ValueRef::List(values) = value {
        return list(context, values, target, field, timestamp_format);
    }
    if let ValueRef::Text(text) = value
        && media_typed(context, target)?
    {
        return encoded(context, text.as_bytes(), field);
    }
    scalar(context, value, field, timestamp_format)
}

fn media_typed(context: RequestContext<'_>, target: Target) -> Result<bool, EncodeError> {
    let Target::Shape(index) = target else {
        return Ok(false);
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    Ok(shape.wire()?.media_type()?.is_some())
}

fn list(
    context: RequestContext<'_>,
    values: ListRef<'_>,
    target: Target,
    field: FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<String, EncodeError> {
    let Target::Shape(index) = target else {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "header list has no list metadata",
        });
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    let element = shape.element()?.ok_or(EncodeError::InvalidValue {
        field: Some(field),
        reason: "header list has no element",
    })?;
    let format = uri::target_timestamp(
        context,
        element,
        timestamp_format.or(shape.element_wire()?.timestamp_format()?),
    )?;
    let media_typed = media_typed(context, element)?;
    let mut output = BoundedBody::new(context.limits.header_bytes());
    for (offset, item) in values.iter()?.enumerate() {
        if offset > 0 {
            output.write(b", ")?;
        }
        match item? {
            ValueRef::Text(text) if media_typed => {
                output.write(encoded(context, text.as_bytes(), field)?.as_bytes())?
            }
            ValueRef::Text(text) => quoted(&mut output, text)?,
            item => output.write(scalar(context, item, field, format)?.as_bytes())?,
        }
    }
    String::from_utf8(output.into_vec()).map_err(|_| EncodeError::InvalidValue {
        field: Some(field),
        reason: "invalid header encoding",
    })
}

fn quoted(output: &mut BoundedBody, text: &str) -> Result<(), EncodeError> {
    if !text.bytes().any(|byte| matches!(byte, b',' | b'"')) {
        return output.write(text.as_bytes());
    }
    output.write(b"\"")?;
    for byte in text.bytes() {
        if matches!(byte, b'"' | b'\\') {
            output.write(b"\\")?;
        }
        output.write(&[byte])?;
    }
    output.write(b"\"")
}

fn encoded(
    context: RequestContext<'_>,
    bytes: &[u8],
    field: FieldId,
) -> Result<String, EncodeError> {
    let mut body = BoundedBody::new(context.limits.header_bytes());
    base64(&mut body, bytes)?;
    String::from_utf8(body.into_vec()).map_err(|_| EncodeError::InvalidValue {
        field: Some(field),
        reason: "invalid base64 header",
    })
}

fn scalar(
    context: RequestContext<'_>,
    value: ValueRef<'_>,
    field: FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<String, EncodeError> {
    match value {
        ValueRef::Bytes(bytes) => encoded(context, bytes, field),
        ValueRef::Text(text) => Ok(text.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_f64().to_string()),
        ValueRef::Time(time) => {
            timestamp::format(time, timestamp_format.or(Some(TimestampFormat::HttpDate)))
        }
        ValueRef::Null => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "null header value",
        }),
        ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "container header value",
        }),
    }
}

pub(super) fn prefix(
    headers: &mut HeaderMap,
    prefix: &str,
    value: ValueRef<'_>,
    field: FieldId,
) -> Result<(), EncodeError> {
    let ValueRef::Map(map) = value else {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "prefix headers must be a map",
        });
    };
    for entry in map.iter()? {
        let (key, value) = entry?;
        let KeyRef::Name(key) = key else {
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "prefix header key must be text",
            });
        };
        let ValueRef::Text(value) = value else {
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "prefix header value must be text",
            });
        };
        let name = format!("{prefix}{key}");
        if !headers.contains_key(name.as_str()) {
            write_header(headers, &name, value)?;
        }
    }
    Ok(())
}
