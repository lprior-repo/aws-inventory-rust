use super::{field_value, EncodeError, RequestContext};
use inventory_core::{catalog::{HttpBinding, Target}, record::ValueRef};

pub(super) fn target(context: RequestContext<'_>) -> Result<String, EncodeError> {
    let http = context.operation.http()?.ok_or(EncodeError::Unsupported("operation without HTTP binding"))?;
    if http.uri.len() > context.limits.uri_bytes() { return Err(EncodeError::Limit("URI")); }
    let mut output = String::new();
    output.try_reserve(http.uri.len()).map_err(EncodeError::Allocation)?;
    output.push_str(http.uri);
    let input = context.operation.input()?;
    if let Target::Shape(index) = input {
        let structure = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
        for member in structure.members()? {
            let member = member?;
            let HttpBinding::Label = member.wire()?.binding()? else { continue };
            let value = match field_value(context, member)? {
                Some(value) => value,
                None if member.wire()?.idempotency_token() => context.idempotency_token.map(ValueRef::Text).ok_or(EncodeError::InvalidValue { field: Some(member.id), reason: "missing URI label" })?,
                None => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "missing URI label" }),
            };
            let name = member.name()?;
            let regular = format!("{{{name}}}");
            let greedy = format!("{{{name}+}}");
            let format = member.wire()?.timestamp_format()?;
            let format = target_timestamp(context, member.target, format)?;
            let replacement = scalar(context, member.target, value, member.id, format)?;
            let encoded = encode_label(&replacement, greedy_match(&output, &greedy), context.limits.uri_bytes())?;
            if output.contains(&greedy) {
                output = replace_label(&output, &greedy, &encoded, context.limits.uri_bytes())?;
            } else if output.contains(&regular) {
                output = replace_label(&output, &regular, &encoded, context.limits.uri_bytes())?;
            } else {
                return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "URI label not found" });
            }
        }
    }
    if output.len() > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    Ok(output)
}

fn greedy_match(path: &str, label: &str) -> bool {
    path.contains(label)
}

fn replace_label(output: &str, marker: &str, replacement: &str, limit: usize) -> Result<String, EncodeError> {
    let start = output.find(marker).ok_or(EncodeError::InvalidEndpoint)?;
    let end = start.checked_add(marker.len()).ok_or(EncodeError::Limit("URI"))?;
    let length = output.len().checked_sub(marker.len())
        .and_then(|prefix| prefix.checked_add(replacement.len()))
        .ok_or(EncodeError::Limit("URI"))?;
    if length > limit { return Err(EncodeError::Limit("URI")); }
    let before = output.get(..start).ok_or(EncodeError::InvalidEndpoint)?;
    let after = output.get(end..).ok_or(EncodeError::InvalidEndpoint)?;
    let mut result = String::new();
    result.try_reserve_exact(length).map_err(EncodeError::Allocation)?;
    result.push_str(before);
    result.push_str(replacement);
    result.push_str(after);
    Ok(result)
}

fn scalar(_context: RequestContext<'_>, _target: Target, value: ValueRef<'_>, field: inventory_core::record::FieldId, format: Option<inventory_core::catalog::TimestampFormat>) -> Result<String, EncodeError> {
    match value {
        ValueRef::Text(value) => Ok(value.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_string()),
        ValueRef::Time(value) => super::timestamp::format(value, format),
        ValueRef::Bytes(_) => Err(EncodeError::InvalidValue { field: Some(field), reason: "blob cannot be used as scalar URI value" }),
        ValueRef::Null => Err(EncodeError::InvalidValue { field: Some(field), reason: "null URI value" }),
        ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue { field: Some(field), reason: "container cannot be used as scalar URI value" }),
    }
}

pub(super) fn target_timestamp(context: RequestContext<'_>, target: Target, member: Option<inventory_core::catalog::TimestampFormat>) -> Result<Option<inventory_core::catalog::TimestampFormat>, EncodeError> {
    match target {
        Target::Shape(index) => {
            let shape = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
            Ok(member.or(shape.wire()?.timestamp_format()?))
        }
        Target::Prelude(_) => Ok(member),
    }
}

pub(super) fn encode_label(value: &str, greedy: bool, limit: usize) -> Result<String, EncodeError> {
    let mut output = String::new();
    for byte in value.bytes() {
        let unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');
        let length = if unreserved || byte == b'/' && greedy { 1 } else { 3 };
        if output.len().checked_add(length).ok_or(EncodeError::Limit("URI"))? > limit {
            return Err(EncodeError::Limit("URI"));
        }
        output.try_reserve(length).map_err(EncodeError::Allocation)?;
        if length == 1 {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(hex(byte >> 4)));
            output.push(char::from(hex(byte & 15)));
        }
    }
    Ok(output)
}

fn hex(value: u8) -> u8 {
    match value {
        0 => b'0', 1 => b'1', 2 => b'2', 3 => b'3',
        4 => b'4', 5 => b'5', 6 => b'6', 7 => b'7',
        8 => b'8', 9 => b'9', 10 => b'A', 11 => b'B',
        12 => b'C', 13 => b'D', 14 => b'E', 15 => b'F',
        _ => b'0',
    }
}

fn append_query(output: &mut Vec<u8>, name: &str, value: &str, limit: usize) -> Result<(), EncodeError> {
    if !output.is_empty() { append_byte(output, b'&', limit)?; }
    append_escaped(output, name, limit)?;
    append_byte(output, b'=', limit)?;
    append_escaped(output, value, limit)
}

fn append_byte(output: &mut Vec<u8>, byte: u8, limit: usize) -> Result<(), EncodeError> {
    if output.len().checked_add(1).ok_or(EncodeError::Limit("URI"))? > limit {
        return Err(EncodeError::Limit("URI"));
    }
    output.try_reserve(1).map_err(EncodeError::Allocation)?;
    output.push(byte);
    Ok(())
}

fn append_escaped(output: &mut Vec<u8>, value: &str, limit: usize) -> Result<(), EncodeError> {
    value.bytes().try_for_each(|byte| {
        let count = if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') { 1 } else { 3 };
        if output.len().checked_add(count).ok_or(EncodeError::Limit("URI"))? > limit {
            return Err(EncodeError::Limit("URI"));
        }
        output.try_reserve(count).map_err(EncodeError::Allocation)?;
        if count == 1 {
            output.push(byte);
        } else {
            output.extend_from_slice(&[b'%', hex(byte >> 4), hex(byte & 15)]);
        }
        Ok(())
    })
}

pub(super) fn append_member_query(
    context: RequestContext<'_>,
    target: &mut String,
    name: &str,
    value: ValueRef<'_>,
    member_target: Target,
    field: inventory_core::record::FieldId,
    timestamp_format: Option<inventory_core::catalog::TimestampFormat>,
) -> Result<(), EncodeError> {
    let format = target_timestamp(context, member_target, timestamp_format)?;
    let value = scalar(context, member_target, value, field, format)?;
    let remaining = context.limits.uri_bytes().checked_sub(target.len())
        .and_then(|length| length.checked_sub(1)).ok_or(EncodeError::Limit("URI"))?;
    let mut query = Vec::new();
    append_query(&mut query, name, &value, remaining)?;
    let length = target.len().checked_add(query.len()).and_then(|length| length.checked_add(1))
        .ok_or(EncodeError::Limit("URI"))?;
    if length > context.limits.uri_bytes() { return Err(EncodeError::Limit("URI")); }
    target.try_reserve(length.checked_sub(target.len()).ok_or(EncodeError::Limit("URI"))?)
        .map_err(EncodeError::Allocation)?;
    target.push(if target.contains('?') { '&' } else { '?' });
    target.push_str(std::str::from_utf8(&query).map_err(|_| EncodeError::InvalidValue {
        field: Some(field),
        reason: "invalid query encoding",
    })?);
    Ok(())
}

pub(super) fn endpoint(context: RequestContext<'_>) -> Result<String, EncodeError> {
    if context.endpoint.len() > context.limits.uri_bytes() { return Err(EncodeError::Limit("URI")); }
    let Some(host_prefix) = context.operation.host_prefix()? else { return Ok(context.endpoint.to_owned()) };
    let input = context.operation.input()?;
    let Target::Shape(index) = input else { return Err(EncodeError::InvalidValue { field: None, reason: "host prefix requires a structure input" }) };
    let structure = context.service.shape(index)?.ok_or(EncodeError::MissingInput)?;
    let mut prefix = host_prefix.to_owned();
    let mut found = false;
    for item in structure.members()? {
        let member = item?;
        if !member.wire()?.host_label() { continue }
        let value = field_value(context, member)?.ok_or(EncodeError::InvalidValue { field: Some(member.id), reason: "missing host label" })?;
        let format = target_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let label = scalar(context, member.target, value, member.id, format)?;
        let marker = format!("{{{}}}", member.name()?);
        prefix = replace_label(&prefix, &marker, &label, context.limits.uri_bytes())?;
        found = true;
    }
    if !found { return Err(EncodeError::InvalidValue { field: None, reason: "host prefix has no host label" }); }
    if prefix.contains('{') || prefix.contains('}') { return Err(EncodeError::InvalidValue { field: None, reason: "unresolved host label" }); }
    prepend_host(context.endpoint, &prefix, context.limits.uri_bytes())
}

fn prepend_host(endpoint: &str, prefix: &str, limit: usize) -> Result<String, EncodeError> {
    let uri = endpoint.parse::<http::Uri>().map_err(|_| EncodeError::InvalidEndpoint)?;
    let scheme = uri.scheme_str().ok_or(EncodeError::InvalidEndpoint)?;
    let authority = uri.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let host = authority.host();
    let port = match authority.port_u16() { Some(port) => format!(":{port}"), None => String::new() };
    let path = uri.path_and_query().map_or("", http::uri::PathAndQuery::as_str);
    let length = scheme.len().checked_add(3).and_then(|size| size.checked_add(prefix.len()))
        .and_then(|size| size.checked_add(host.len())).and_then(|size| size.checked_add(port.len()))
        .and_then(|size| size.checked_add(path.len())).ok_or(EncodeError::Limit("URI"))?;
    if length > limit { return Err(EncodeError::Limit("URI")); }
    let mut result = String::new();
    result.try_reserve_exact(length).map_err(EncodeError::Allocation)?;
    result.push_str(scheme);
    result.push_str("://");
    result.push_str(prefix);
    result.push_str(host);
    result.push_str(&port);
    result.push_str(path);
    Ok(result)
}
