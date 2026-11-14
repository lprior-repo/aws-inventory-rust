use super::super::super::body::json as scalar_json;
use super::super::super::common::BoundedBody;
use super::super::EncodeError;
use super::super::super::RequestContext;
use inventory_core::{catalog::{HttpBinding, Shape, Target, TimestampFormat}, record::{ListRef, MapRef, ValueRef}};

const MAX_ITEMS: usize = 65_536;

#[derive(Clone, Copy)]
enum Task<'a> {
    Value(Target, ValueRef<'a>, Option<&'a str>, Option<TimestampFormat>, usize),
    Name(&'a str),
    Raw(&'static [u8]),
}

pub(super) fn encode(context: RequestContext<'_>, input: Shape<'_>) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    write_structure(context, &mut body, &mut tasks, input, context.input, 0)?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}

pub(super) fn encode_payload(context: RequestContext<'_>, target: Target, value: ValueRef<'_>) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    let format = effective_timestamp(context, target, None)?;
    write_value(context, &mut body, &mut tasks, target, value, None, format, 0)?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}

fn write_tasks(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>) -> Result<(), EncodeError> {
    while let Some(task) = tasks.pop() {
        match task {
            Task::Raw(bytes) => body.write(bytes)?,
            Task::Name(name) => write_name(body, name)?,
            Task::Value(target, value, name, format, depth) => write_value(context, body, tasks, target, value, name, format, depth)?,
        }
    }
    Ok(())
}

fn write_value(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, value: ValueRef<'_>, name: Option<&str>, format: Option<TimestampFormat>, depth: usize) -> Result<(), EncodeError> {
    if depth > inventory_core::record::MAX_DEPTH { return Err(EncodeError::Limit("frames")); }
    if let Some(name) = name { write_name(body, name)?; body.write(b":")?; }
    match value {
        ValueRef::Null => body.write(b"null")?,
        ValueRef::Bool(value) => body.write(if value { b"true" } else { b"false" })?,
        ValueRef::Int(value) => scalar_json::integer(body, value)?,
        ValueRef::Float(value) => scalar_json::float(body, value.to_f64())?,
        ValueRef::Text(value) => scalar_json::string(body, value)?,
        ValueRef::Bytes(value) => write_bytes(body, value)?,
        ValueRef::Time(value) => write_timestamp(body, value, format)?,
        ValueRef::List(list) => write_list(context, body, tasks, target, list, depth)?,
        ValueRef::Map(map) => write_map(context, body, tasks, target, map, depth)?,
    }
    Ok(())
}

fn write_bytes(body: &mut BoundedBody, bytes: &[u8]) -> Result<(), EncodeError> {
    body.write(b"\"")?;
    scalar_json::base64(body, bytes)?;
    body.write(b"\"")
}

fn write_timestamp(body: &mut BoundedBody, time: inventory_core::record::Time, format: Option<TimestampFormat>) -> Result<(), EncodeError> {
    let text = super::super::timestamp::format(time, format)?;
    if matches!(format.unwrap_or(TimestampFormat::DateTime), TimestampFormat::EpochSeconds) { return body.write(text.as_bytes()); }
    body.write(b"\"")?;
    scalar_json::string(body, &text)?;
    body.write(b"\"")
}

fn write_name(body: &mut BoundedBody, name: &str) -> Result<(), EncodeError> {
    scalar_json::string(body, name)
}

fn write_list(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, list: ListRef<'_>, depth: usize) -> Result<(), EncodeError> {
    let current = shape(context, target)?;
    let element = current.element()?.ok_or(EncodeError::Unsupported("list without element"))?;
    let element_wire = current.element_wire()?;
    let format = effective_timestamp(context, element, element_wire.timestamp_format()?)?;
    if element_wire.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
    let mut values = Vec::new();
    for value in list.iter()? {
        if values.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        let value = value?;
        if matches!(value, ValueRef::Null) && !element_wire.sparse() { return Err(EncodeError::InvalidValue { field: None, reason: "null in non-sparse list" }); }
        values.try_reserve(1).map_err(EncodeError::Allocation)?;
        values.push(value);
    }
    reserve_tasks(tasks, values.len().saturating_mul(2).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"]"));
    values.iter().rev().enumerate().try_for_each(|(offset, value)| {
        if offset > 0 { tasks.push(Task::Raw(b",")); }
        tasks.push(Task::Value(element, *value, None, format, next));
        Ok::<(), EncodeError>(())
    })?;
    body.write(b"[")
}

fn write_map(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, map: MapRef<'_>, depth: usize) -> Result<(), EncodeError> {
    let current = shape(context, target)?;
    if matches!(current.kind()?, "structure" | "union") { return write_structure(context, body, tasks, current, map, depth); }
    let value_target = current.map_value()?.ok_or(EncodeError::Unsupported("map without value"))?;
    let value_wire = current.map_value_wire()?;
    if current.map_key_wire()?.streaming() || current.map_value_wire()?.streaming() {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let format = effective_timestamp(context, value_target, value_wire.timestamp_format()?)?;
    let mut entries = Vec::new();
    for entry in map.iter()? {
        if entries.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        let entry = entry?;
        if matches!(entry.1, ValueRef::Null) && !value_wire.sparse() { return Err(EncodeError::InvalidValue { field: None, reason: "null in non-sparse map" }); }
        entries.try_reserve(1).map_err(EncodeError::Allocation)?;
        entries.push(entry);
    }
    reserve_tasks(tasks, entries.len().saturating_mul(3).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"}"));
    entries.iter().rev().enumerate().try_for_each(|(offset, (key, value))| {
        let inventory_core::record::KeyRef::Name(name) = key else { return Err(EncodeError::InvalidValue { field: None, reason: "map key is not text" }) };
        if offset > 0 { tasks.push(Task::Raw(b",")); }
        tasks.push(Task::Value(value_target, *value, None, format, next));
        tasks.push(Task::Name(name));
        Ok(())
    })?;
    body.write(b"{")
}

fn write_structure(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, structure: Shape<'_>, map: MapRef<'_>, depth: usize) -> Result<(), EncodeError> {
    let mut members = Vec::new();
    for member in structure.members()? {
        if members.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        members.try_reserve(1).map_err(EncodeError::Allocation)?;
        members.push(member?);
    }
    let union = structure.kind()? == "union";
    let mut present = 0_usize;
    let mut fields = Vec::new();
    for member in &members {
        let value = match map.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token() => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required member is missing" }),
                None => continue,
            },
            None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required member is missing" }),
            None => continue,
        };
        if member.wire()?.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
        if matches!(value, ValueRef::Null) {
            if member.required { return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required member is null" }); }
            continue;
        }
        if !matches!(member.wire()?.binding()?, HttpBinding::Document) { continue; }
        present = present.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
        fields.try_reserve(1).map_err(EncodeError::Allocation)?;
        fields.push((*member, value));
    }
    if union && present != 1 { return Err(EncodeError::InvalidValue { field: None, reason: "JSON union must contain exactly one variant" }); }
    reserve_tasks(tasks, fields.len().saturating_mul(3).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"}"));
    fields.iter().rev().enumerate().try_for_each(|(offset, (member, value))| {
        if offset > 0 { tasks.push(Task::Raw(b",")); }
        let field_name = match member.wire()?.json_name()? { Some(name) => name, None => context.service.field_name(member.id)? };
        let format = effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        tasks.push(Task::Value(member.target, *value, Some(field_name), format, next));
        Ok::<(), EncodeError>(())
    })?;
    body.write(b"{")
}

fn effective_timestamp(context: RequestContext<'_>, target: Target, member: Option<TimestampFormat>) -> Result<Option<TimestampFormat>, EncodeError> {
    match target {
        Target::Shape(index) => Ok(member.or(context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.wire()?.timestamp_format()?)),
        Target::Prelude(_) => Ok(member),
    }
}

fn shape(context: RequestContext<'_>, target: Target) -> Result<Shape<'_>, EncodeError> {
    match target {
        Target::Shape(index) => context.service.shape(index)?.ok_or(EncodeError::MissingInput),
        Target::Prelude(_) => Err(EncodeError::Unsupported("scalar has no shape metadata")),
    }
}

fn reserve_tasks(tasks: &mut Vec<Task<'_>>, additional: usize) -> Result<(), EncodeError> {
    if tasks.len().saturating_add(additional) > 65_536 { return Err(EncodeError::Limit("frames")); }
    tasks.try_reserve(additional).map_err(EncodeError::Allocation)
}
