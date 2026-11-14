use super::{field_value, uri, write_header, EncodeError, RequestContext};
use http::header::HeaderMap;
use inventory_core::{catalog::HttpBinding, record::ValueRef};

pub(super) fn headers(context: RequestContext<'_>, target: &mut String, headers: &mut HeaderMap) -> Result<(), EncodeError> {
    let input = context.operation.input()?;
    let inventory_core::catalog::Target::Shape(index) = input else { return Ok(()) };
    let structure = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
    for member in structure.members()? {
        let member = member?;
        let value = match field_value(context, member)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token() => match context.idempotency_token {
                Some(value) => ValueRef::Text(value),
                None => continue,
            },
            None => continue,
        };
        match member.wire()?.binding()? {
            HttpBinding::Header(header) => {
                let format = uri::target_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
                write_header(headers, header, &header_value(context, value, member.target, member.id, format)?)?
            }
            HttpBinding::PrefixHeaders(prefix) => prefix_headers(headers, prefix, value, member.id)?,
            HttpBinding::Query(query) => uri::append_member_query(context, target, query, value, member.target, member.id, member.wire()?.timestamp_format()?)?,
            HttpBinding::QueryParams => query_params(context, target, value, member.id)?,
            HttpBinding::Payload | HttpBinding::ResponseCode | HttpBinding::Label | HttpBinding::Document => {}
        }
    }
    Ok(())
}

fn header_value(context: RequestContext<'_>, value: ValueRef<'_>, target: inventory_core::catalog::Target, field: inventory_core::record::FieldId, timestamp_format: Option<inventory_core::catalog::TimestampFormat>) -> Result<String, EncodeError> {
    if let ValueRef::List(values) = value { return header_list(context, values, target, field, timestamp_format); }
    header_scalar(context, value, field, timestamp_format)
}

fn header_list(context: RequestContext<'_>, values: inventory_core::record::ListRef<'_>, target: inventory_core::catalog::Target, field: inventory_core::record::FieldId, timestamp_format: Option<inventory_core::catalog::TimestampFormat>) -> Result<String, EncodeError> {
    let inventory_core::catalog::Target::Shape(index) = target else {
        return Err(EncodeError::InvalidValue { field: Some(field), reason: "header list has no list metadata" });
    };
    let shape = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
    let element = shape.element()?.ok_or(EncodeError::InvalidValue { field: Some(field), reason: "header list has no element" })?;
    let format = uri::target_timestamp(context, element, timestamp_format.or(shape.element_wire()?.timestamp_format()?))?;
    let mut output = super::super::common::BoundedBody::new(context.limits.header_bytes());
    for (offset, item) in values.iter()?.enumerate() {
        if offset > 0 { output.write(b",")?; }
        let text = header_scalar(context, item?, field, format)?;
        output.write(text.as_bytes())?;
    }
    String::from_utf8(output.into_vec()).map_err(|_| EncodeError::InvalidValue { field: Some(field), reason: "invalid header encoding" })
}

fn header_scalar(context: RequestContext<'_>, value: ValueRef<'_>, field: inventory_core::record::FieldId, timestamp_format: Option<inventory_core::catalog::TimestampFormat>) -> Result<String, EncodeError> {
    match value {
        ValueRef::Bytes(bytes) => {
            let mut body = super::super::common::BoundedBody::new(context.limits.header_bytes());
            super::super::body::json::base64(&mut body, bytes)?;
            String::from_utf8(body.into_vec()).map_err(|_| EncodeError::InvalidValue { field: Some(field), reason: "invalid base64 header" })
        }
        ValueRef::Text(text) => Ok(text.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_string()),
        ValueRef::Time(time) => super::timestamp::format(time, timestamp_format),
        ValueRef::Null => Err(EncodeError::InvalidValue { field: Some(field), reason: "null header value" }),
        ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue { field: Some(field), reason: "container header value" }),
    }
}

fn prefix_headers(headers: &mut HeaderMap, prefix: &str, value: ValueRef<'_>, field: inventory_core::record::FieldId) -> Result<(), EncodeError> {
    let ValueRef::Map(map) = value else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "prefix headers must be a map" }) };
    for entry in map.iter()? {
        let (key, value) = entry?;
        let inventory_core::record::KeyRef::Name(key) = key else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "prefix header key must be text" }) };
        let ValueRef::Text(value) = value else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "prefix header value must be text" }) };
        let name = format!("{prefix}{key}");
        write_header(headers, &name, value)?;
    }
    Ok(())
}

fn query_params(context: RequestContext<'_>, target: &mut String, value: ValueRef<'_>, field: inventory_core::record::FieldId) -> Result<(), EncodeError> {
    let ValueRef::Map(map) = value else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "query parameters must be a map" }) };
    for entry in map.iter()? {
        let (key, value) = entry?;
        let inventory_core::record::KeyRef::Name(key) = key else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "query parameter key must be text" }) };
        let ValueRef::List(values) = value else { return Err(EncodeError::InvalidValue { field: Some(field), reason: "query parameter values must be a list" }) };
        for item in values.iter()? {
            uri::append_member_query(context, target, key, item?, inventory_core::catalog::Target::Prelude("String"), field, None)?;
        }
    }
    Ok(())
}

