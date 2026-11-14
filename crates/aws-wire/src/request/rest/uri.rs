use super::{EncodeError, RequestContext, field_value};
use inventory_core::{
    catalog::{HttpBinding, Target},
    record::ValueRef,
};
pub(super) mod query;

pub(super) fn target(context: RequestContext<'_>) -> Result<String, EncodeError> {
    let http = context
        .operation
        .http()?
        .ok_or(EncodeError::Unsupported("operation without HTTP binding"))?;
    if http.uri.len() > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    let mut output = String::new();
    output
        .try_reserve(http.uri.len())
        .map_err(EncodeError::Allocation)?;
    output.push_str(http.uri);
    if let Some(addressing) = context.addressing {
        output = addressed_path(&output, addressing.path_label, context.limits.uri_bytes())?;
    }
    let input = context.operation.input()?;
    if let Target::Shape(index) = input {
        let structure = context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?;
        for member in structure.members()? {
            let member = member?;
            let HttpBinding::Label = member.wire()?.binding()? else {
                continue;
            };
            if let Some(addressing) = context.addressing
                && member.name()? == addressing.path_label
            {
                continue;
            }
            let supplied = field_value(context, member)?;
            let supplied = crate::request::customization::label(context, member.name()?, supplied)?;
            let value = match supplied {
                Some(value) => value,
                None if member.wire()?.idempotency_token()? => context
                    .idempotency_token
                    .map(ValueRef::Text)
                    .ok_or(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "missing URI label",
                    })?,
                None => {
                    return Err(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "missing URI label",
                    });
                }
            };
            let name = member.name()?;
            let regular = format!("{{{name}}}");
            let greedy = format!("{{{name}+}}");
            let format = member.wire()?.timestamp_format()?;
            let format = target_timestamp(context, member.target, format)?;
            let replacement = scalar(context, member.target, value, member.id, format)?;
            let encoded = encode_label(
                &replacement,
                greedy_match(&output, &greedy),
                context.limits.uri_bytes(),
            )?;
            if output.contains(&greedy) {
                output = replace_label(&output, &greedy, &encoded, context.limits.uri_bytes())?;
            } else if output.contains(&regular) {
                output = replace_label(&output, &regular, &encoded, context.limits.uri_bytes())?;
            } else {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "URI label not found",
                });
            }
        }
    }
    if output.len() > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    Ok(output)
}

fn addressed_path(template: &str, label: &str, limit: usize) -> Result<String, EncodeError> {
    let marker = format!("/{{{label}}}");
    let (prefix, suffix) = template
        .split_once(&marker)
        .filter(|(_, suffix)| suffix.is_empty() || suffix.starts_with(['/', '?']))
        .ok_or(EncodeError::Unsupported("addressing path label not found"))?;
    let mut output = String::new();
    output.try_reserve_exact(template.len().min(limit))?;
    output.push_str(prefix);
    if output.is_empty() && !suffix.starts_with('/') {
        output.push('/');
    }
    output.push_str(suffix);
    Ok(output)
}

fn greedy_match(path: &str, label: &str) -> bool {
    path.contains(label)
}

fn replace_label(
    output: &str,
    marker: &str,
    replacement: &str,
    limit: usize,
) -> Result<String, EncodeError> {
    let start = output.find(marker).ok_or(EncodeError::InvalidEndpoint)?;
    let end = start
        .checked_add(marker.len())
        .ok_or(EncodeError::Limit("URI"))?;
    let length = output
        .len()
        .checked_sub(marker.len())
        .and_then(|prefix| prefix.checked_add(replacement.len()))
        .ok_or(EncodeError::Limit("URI"))?;
    if length > limit {
        return Err(EncodeError::Limit("URI"));
    }
    let before = output.get(..start).ok_or(EncodeError::InvalidEndpoint)?;
    let after = output.get(end..).ok_or(EncodeError::InvalidEndpoint)?;
    let mut result = String::new();
    result
        .try_reserve_exact(length)
        .map_err(EncodeError::Allocation)?;
    result.push_str(before);
    result.push_str(replacement);
    result.push_str(after);
    Ok(result)
}

fn is_float_target(context: RequestContext<'_>, target: Target) -> Result<bool, EncodeError> {
    let kind = match target {
        Target::Prelude(kind) => kind,
        Target::Shape(index) => context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .kind()?,
        _ => return Err(EncodeError::Unsupported("catalog target kind")),
    };
    Ok(matches!(kind, "float" | "double"))
}

fn scalar(
    context: RequestContext<'_>,
    target: Target,
    value: ValueRef<'_>,
    field: inventory_core::record::FieldId,
    format: Option<inventory_core::catalog::TimestampFormat>,
) -> Result<String, EncodeError> {
    if is_float_target(context, target)?
        && matches!(value, ValueRef::Text(text) if !matches!(text, "NaN" | "Infinity" | "-Infinity"))
    {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "invalid float sentinel",
        });
    }
    match value {
        ValueRef::Text(value) => Ok(value.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_f64().to_string()),
        ValueRef::Time(value) => super::timestamp::format(value, format),
        ValueRef::Bytes(_) => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "blob cannot be used as scalar URI value",
        }),
        ValueRef::Null => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "null URI value",
        }),
        ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "container cannot be used as scalar URI value",
        }),
    }
}

pub(super) fn target_timestamp(
    context: RequestContext<'_>,
    target: Target,
    member: Option<inventory_core::catalog::TimestampFormat>,
) -> Result<Option<inventory_core::catalog::TimestampFormat>, EncodeError> {
    match target {
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(EncodeError::MissingInput)?;
            Ok(member.or(shape.wire()?.timestamp_format()?))
        }
        Target::Prelude(_) => Ok(member),
        _ => Err(EncodeError::Unsupported("unknown timestamp target")),
    }
}

pub(super) fn encode_label(value: &str, greedy: bool, limit: usize) -> Result<String, EncodeError> {
    let mut output = String::new();
    for byte in value.bytes() {
        let unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');
        let length = if unreserved || byte == b'/' && greedy {
            1
        } else {
            3
        };
        if output
            .len()
            .checked_add(length)
            .ok_or(EncodeError::Limit("URI"))?
            > limit
        {
            return Err(EncodeError::Limit("URI"));
        }
        output
            .try_reserve(length)
            .map_err(EncodeError::Allocation)?;
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
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'A',
        11 => b'B',
        12 => b'C',
        13 => b'D',
        14 => b'E',
        15 => b'F',
        _ => b'0',
    }
}
