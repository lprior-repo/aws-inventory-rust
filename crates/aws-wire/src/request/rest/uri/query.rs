use super::{EncodeError, RequestContext, hex, scalar, target_timestamp};
use inventory_core::{
    catalog::{Target, TimestampFormat},
    record::{ListRef, ValueRef},
};

fn append_query(
    output: &mut Vec<u8>,
    name: &str,
    value: &str,
    limit: usize,
) -> Result<(), EncodeError> {
    if !output.is_empty() {
        append_byte(output, b'&', limit)?;
    }
    append_escaped(output, name, limit)?;
    append_byte(output, b'=', limit)?;
    append_escaped(output, value, limit)
}

fn append_byte(output: &mut Vec<u8>, byte: u8, limit: usize) -> Result<(), EncodeError> {
    if output
        .len()
        .checked_add(1)
        .ok_or(EncodeError::Limit("URI"))?
        > limit
    {
        return Err(EncodeError::Limit("URI"));
    }
    output.try_reserve(1).map_err(EncodeError::Allocation)?;
    output.push(byte);
    Ok(())
}

fn append_escaped(output: &mut Vec<u8>, value: &str, limit: usize) -> Result<(), EncodeError> {
    value.bytes().try_for_each(|byte| {
        let count = if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            1
        } else {
            3
        };
        if output
            .len()
            .checked_add(count)
            .ok_or(EncodeError::Limit("URI"))?
            > limit
        {
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

pub(in crate::request::rest) const MAX_QUERY_ITEMS: usize = 65_536;

pub(in crate::request::rest) fn append_member_query(
    context: RequestContext<'_>,
    target: &mut String,
    name: &str,
    value: ValueRef<'_>,
    member_target: Target,
    field: inventory_core::record::FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<(), EncodeError> {
    match value {
        ValueRef::List(list) => append_list_query(
            context,
            target,
            name,
            list,
            member_target,
            field,
            timestamp_format,
        ),
        value => append_scalar_query(
            context,
            target,
            name,
            value,
            member_target,
            field,
            timestamp_format,
        ),
    }
}

fn append_list_query(
    context: RequestContext<'_>,
    target: &mut String,
    name: &str,
    list: ListRef<'_>,
    member_target: Target,
    field: inventory_core::record::FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<(), EncodeError> {
    let (element, format, sparse) = list_element(context, member_target, timestamp_format)?;
    for (index, item) in list.iter()?.enumerate() {
        if index == MAX_QUERY_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let item = item?;
        if matches!(item, ValueRef::Null) {
            if sparse {
                continue;
            }
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "null in non-sparse query list",
            });
        }
        append_scalar_query(context, target, name, item, element, field, format)?;
    }
    Ok(())
}

fn list_element(
    context: RequestContext<'_>,
    target: Target,
    timestamp_format: Option<TimestampFormat>,
) -> Result<(Target, Option<TimestampFormat>, bool), EncodeError> {
    let Target::Shape(index) = target else {
        return Err(EncodeError::Unsupported("query list without shape"));
    };
    let list = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    let element = list
        .element()?
        .ok_or(EncodeError::Unsupported("query list without element"))?;
    let wire = list.element_wire()?;
    if list.wire()?.streaming()? || wire.streaming()? {
        return Err(EncodeError::Unsupported("streaming query list"));
    }
    Ok((
        element,
        timestamp_format.or(wire.timestamp_format()?),
        wire.sparse()?,
    ))
}

fn append_scalar_query(
    context: RequestContext<'_>,
    target: &mut String,
    name: &str,
    value: ValueRef<'_>,
    member_target: Target,
    field: inventory_core::record::FieldId,
    timestamp_format: Option<TimestampFormat>,
) -> Result<(), EncodeError> {
    let format = target_timestamp(context, member_target, timestamp_format)?;
    let value = scalar(context, member_target, value, field, format)?;
    append_encoded_query(context, target, name, &value, field)
}

fn append_encoded_query(
    context: RequestContext<'_>,
    target: &mut String,
    name: &str,
    value: &str,
    field: inventory_core::record::FieldId,
) -> Result<(), EncodeError> {
    let remaining = context
        .limits
        .uri_bytes()
        .checked_sub(target.len())
        .and_then(|length| length.checked_sub(1))
        .ok_or(EncodeError::Limit("URI"))?;
    let mut query = Vec::new();
    append_query(&mut query, name, value, remaining)?;
    append_query_target(context, target, &query, field)
}

fn append_query_target(
    context: RequestContext<'_>,
    target: &mut String,
    query: &[u8],
    field: inventory_core::record::FieldId,
) -> Result<(), EncodeError> {
    let length = target
        .len()
        .checked_add(query.len())
        .and_then(|length| length.checked_add(1))
        .ok_or(EncodeError::Limit("URI"))?;
    if length > context.limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    target
        .try_reserve(
            length
                .checked_sub(target.len())
                .ok_or(EncodeError::Limit("URI"))?,
        )
        .map_err(EncodeError::Allocation)?;
    target.push(if target.contains('?') { '&' } else { '?' });
    target.push_str(
        std::str::from_utf8(query).map_err(|_| EncodeError::InvalidValue {
            field: Some(field),
            reason: "invalid query encoding",
        })?,
    );
    Ok(())
}
