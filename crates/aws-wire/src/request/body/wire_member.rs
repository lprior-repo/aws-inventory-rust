use super::wire::{
    Encoder, MAX_ITEMS, Task, ValueTask, push, timestamp_format_name, value_is_null,
};
use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::{
    catalog::{Shape, Target},
    record::{FieldId, MapRef, ValueRef},
};

struct CollectedFields<'a> {
    supplied_count: usize,
    fields: Vec<(&'a str, ValueTask<'a>)>,
}
struct MemberData<'a> {
    id: FieldId,
    name: &'a str,
    required: bool,
    target: Target,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
    sparse: bool,
    idempotency_token: bool,
    streaming: bool,
    default_value: Option<ValueRef<'a>>,
    client_optional: bool,
}

pub(super) fn push_structure<'a>(
    encoder: &mut Encoder<'a, '_>,
    shape: Shape<'a>,
    values: MapRef<'a>,
    union: bool,
    parent: ValueTask<'a>,
) -> Result<(), EncodeError> {
    let fields = collect_structure_fields(encoder, shape, values, union, parent)?;
    write_structure_fields(encoder.body, encoder.protocol, fields, encoder.pending)
}

fn collect_structure_fields<'a>(
    encoder: &Encoder<'a, '_>,
    shape: Shape<'a>,
    values: MapRef<'a>,
    union: bool,
    parent: ValueTask<'a>,
) -> Result<Vec<(&'a str, ValueTask<'a>)>, EncodeError> {
    let supplied = count_supplied(values)?;
    let members = collect_members(shape)?;
    let collected = collect_fields(&members, values, union, encoder.idempotency_token, parent)?;
    if collected.supplied_count != supplied {
        return Err(EncodeError::InvalidValue {
            field: parent.field,
            reason: "record contains an unknown member",
        });
    }
    if union && collected.fields.len() != 1 {
        return Err(EncodeError::InvalidValue {
            field: parent.field,
            reason: "union must contain exactly one member",
        });
    }
    Ok(collected.fields)
}

fn count_supplied(values: MapRef<'_>) -> Result<usize, EncodeError> {
    let mut count = 0usize;
    for entry in values.iter()? {
        entry?;
        count = count
            .checked_add(1)
            .ok_or(EncodeError::Limit("container items"))?;
        if count > MAX_ITEMS {
            return Err(EncodeError::Limit("container items"));
        }
    }
    Ok(count)
}

fn collect_members<'a>(shape: Shape<'a>) -> Result<Vec<MemberData<'a>>, EncodeError> {
    let mut members = Vec::new();
    for member in shape.members()? {
        let member = member?;
        let name = member.name()?;
        let wire = member.wire()?;
        members.try_reserve(1)?;
        members.push(MemberData {
            id: member.id,
            name,
            required: member.required,
            target: member.target,
            timestamp_format: wire.timestamp_format()?.map(timestamp_format_name),
            default_value: member.wire()?.default_value()?,
            client_optional: member.wire()?.client_optional()?,
            media_type: wire.media_type()?,
            sparse: wire.sparse()?,
            idempotency_token: wire.idempotency_token()?,
            streaming: wire.streaming()?,
        });
    }
    Ok(members)
}

fn collect_fields<'a>(
    members: &[MemberData<'a>],
    values: MapRef<'a>,
    union: bool,
    token: Option<&'a str>,
    parent: ValueTask<'a>,
) -> Result<CollectedFields<'a>, EncodeError> {
    let mut supplied_count = 0usize;
    let mut fields = Vec::new();
    for member in members {
        let (value, supplied) = member_value(member, values, union, token, parent.depth > 0)?;
        if supplied {
            supplied_count = supplied_count
                .checked_add(1)
                .ok_or(EncodeError::Limit("container items"))?;
        }
        if let Some(value) = value {
            fields.try_reserve(1)?;
            fields.push((member.name, member_task(member, value, parent)?));
        }
    }
    Ok(CollectedFields {
        supplied_count,
        fields,
    })
}

fn member_value<'a>(
    member: &MemberData<'a>,
    values: MapRef<'a>,
    union: bool,
    token: Option<&'a str>,
    nested: bool,
) -> Result<(Option<ValueRef<'a>>, bool), EncodeError> {
    let supplied_value = supplied_member_value(member, values)?;
    let value = match (supplied_value, member.idempotency_token, token) {
        (Some(value), true, Some(token)) if value_is_null(value) => Some(ValueRef::Text(token)),
        (Some(value), _, _) => Some(value),
        (None, true, Some(token)) => Some(ValueRef::Text(token)),
        (None, _, _) if nested && !union && !member.client_optional => member.default_value,
        (None, _, _) => None,
    };
    validate_member_value(member, value, union)?;
    Ok((
        value.filter(|value| !value_is_null(*value)),
        supplied_value.is_some(),
    ))
}

fn supplied_member_value<'a>(
    member: &MemberData<'a>,
    values: MapRef<'a>,
) -> Result<Option<ValueRef<'a>>, EncodeError> {
    let by_id = values.get_field(member.id)?;
    let by_name = values.get_name(member.name)?;
    if by_id.is_some() && by_name.is_some() {
        return Err(EncodeError::InvalidValue {
            field: Some(member.id),
            reason: "member supplied by both field ID and name",
        });
    }
    let value = by_id.or(by_name);
    if value.is_some() && member.streaming {
        return Err(EncodeError::Unsupported("streaming member"));
    }
    Ok(value)
}

fn validate_member_value(
    member: &MemberData<'_>,
    value: Option<ValueRef<'_>>,
    union: bool,
) -> Result<(), EncodeError> {
    if value.is_none() && member.required {
        return Err(EncodeError::InvalidValue {
            field: Some(member.id),
            reason: "required member is missing",
        });
    }
    if value.is_some_and(value_is_null) && (union || member.required) {
        return Err(EncodeError::InvalidValue {
            field: Some(member.id),
            reason: "null in a required or union member",
        });
    }
    Ok(())
}

fn member_task<'a>(
    member: &MemberData<'a>,
    value: ValueRef<'a>,
    parent: ValueTask<'a>,
) -> Result<ValueTask<'a>, EncodeError> {
    let depth = parent
        .depth
        .checked_add(1)
        .ok_or(EncodeError::Limit("serialization depth"))?;
    Ok(ValueTask {
        target: member.target,
        value,
        field: Some(member.id),
        timestamp_format: member.timestamp_format,
        media_type: member.media_type,
        sparse: member.sparse,
        depth,
    })
}

fn write_structure_fields<'a>(
    body: &mut BoundedBody,
    protocol: Protocol,
    fields: Vec<(&'a str, ValueTask<'a>)>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        return write_cbor_fields(body, fields, pending);
    }
    write_json_fields(body, fields, pending)
}

fn write_cbor_fields<'a>(
    body: &mut BoundedBody,
    fields: Vec<(&'a str, ValueTask<'a>)>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    super::cbor::map_indefinite(body)?;
    push(pending, Task::Raw(b"\xff"))?;
    for (name, value) in fields.into_iter().rev() {
        push(pending, Task::Value(value))?;
        push(pending, Task::Key(name))?;
    }
    Ok(())
}

fn write_json_fields<'a>(
    body: &mut BoundedBody,
    fields: Vec<(&'a str, ValueTask<'a>)>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    body.write(b"{")?;
    push(pending, Task::Raw(b"}"))?;
    let length = fields.len();
    for (index, (name, value)) in fields.into_iter().enumerate().rev() {
        if index < length.saturating_sub(1) {
            push(pending, Task::Raw(b","))?;
        }
        push(pending, Task::Value(value))?;
        push(pending, Task::Raw(b":"))?;
        push(pending, Task::Key(name))?;
    }
    Ok(())
}
