use super::super::super::RequestContext;
use super::super::super::body::json as scalar_json;
use super::super::super::common::BoundedBody;
use super::super::EncodeError;
use inventory_core::{
    catalog::{HttpBinding, Shape, Target, TimestampFormat},
    record::{MapRef, ValueRef},
};

const MAX_ITEMS: usize = 65_536;

mod collections;
mod scalar;
pub(super) enum Task<'a> {
    Value(
        Target,
        ValueRef<'a>,
        Option<&'a str>,
        Option<TimestampFormat>,
        usize,
    ),
    Name(&'a str),
    Raw(&'static [u8]),
}

pub(super) fn encode<'a>(
    context: RequestContext<'a>,
    input: Shape<'a>,
) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    write_structure(context, &mut body, &mut tasks, input, context.input, 0)?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}

pub(super) fn encode_payload<'a>(
    context: RequestContext<'a>,
    target: Target,
    value: ValueRef<'a>,
) -> Result<Vec<u8>, EncodeError> {
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    let format = effective_timestamp(context, target, None)?;
    write_value(
        context,
        &mut body,
        &mut tasks,
        (target, value),
        None,
        format,
        0,
    )?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}

fn write_tasks<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    while let Some(task) = tasks.pop() {
        match task {
            Task::Raw(bytes) => body.write(bytes)?,
            Task::Name(name) => scalar::write_name(body, name)?,
            Task::Value(target, value, name, format, depth) => {
                write_value(context, body, tasks, (target, value), name, format, depth)?
            }
        }
    }
    Ok(())
}

fn write_value<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    modeled: (Target, ValueRef<'a>),
    name: Option<&'a str>,
    format: Option<TimestampFormat>,
    depth: usize,
) -> Result<(), EncodeError> {
    let (target, value) = modeled;
    if depth > inventory_core::record::MAX_DEPTH {
        return Err(EncodeError::Limit("frames"));
    }
    if let Some(name) = name {
        scalar::write_name(body, name)?;
        body.write(b":")?;
    }
    match value {
        ValueRef::Null => body.write(b"null")?,
        ValueRef::Bool(value) => body.write(if value { b"true" } else { b"false" })?,
        ValueRef::Int(value) => scalar_json::integer(body, value)?,
        ValueRef::Float(value) => scalar_json::float(body, value.to_f64())?,
        ValueRef::Text(value) => scalar_json::string(body, value)?,
        ValueRef::Bytes(value) => scalar::write_bytes(body, value)?,
        ValueRef::Time(value) => scalar::write_timestamp(body, value, format)?,
        ValueRef::List(list) => collections::write_list(context, body, tasks, target, list, depth)?,
        ValueRef::Map(map) => collections::write_map(context, body, tasks, target, map, depth)?,
    }
    Ok(())
}

fn write_structure<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    structure: Shape<'a>,
    map: MapRef<'a>,
    depth: usize,
) -> Result<(), EncodeError> {
    let mut members = Vec::new();
    for member in structure.members()? {
        if members.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        members.try_reserve(1).map_err(EncodeError::Allocation)?;
        members.push(member?);
    }
    let union = structure.kind()? == "union";
    let mut present = 0_usize;
    let mut fields = Vec::new();
    for member in &members {
        let value = match map.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token()? => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => {
                    return Err(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "required member is missing",
                    });
                }
                None => continue,
            },
            None if depth > 0
                && !union
                && !member.wire()?.client_optional()?
                && let Some(value) = member.wire()?.default_value()?
                && !matches!(value, ValueRef::Null) =>
            {
                value
            }
            None if member.required => {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required member is missing",
                });
            }
            None => continue,
        };
        if member.wire()?.streaming()? {
            return Err(EncodeError::Unsupported("streaming request body"));
        }
        if matches!(value, ValueRef::Null) {
            if member.required {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required member is null",
                });
            }
            continue;
        }
        if !matches!(member.wire()?.binding()?, HttpBinding::Document) {
            continue;
        }
        present = present.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
        fields.try_reserve(1).map_err(EncodeError::Allocation)?;
        fields.push((*member, value));
    }
    if union && present != 1 {
        return Err(EncodeError::InvalidValue {
            field: None,
            reason: "JSON union must contain exactly one variant",
        });
    }
    reserve_tasks(tasks, fields.len().saturating_mul(3).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"}"));
    fields
        .iter()
        .rev()
        .enumerate()
        .try_for_each(|(offset, (member, value))| {
            if offset > 0 {
                tasks.push(Task::Raw(b","));
            }
            let field_name = match member.wire()?.json_name()? {
                Some(name) => name,
                None => context.service.field_name(member.id)?,
            };
            let format =
                effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
            tasks.push(Task::Value(
                member.target,
                *value,
                Some(field_name),
                format,
                next,
            ));
            Ok::<(), EncodeError>(())
        })?;
    body.write(b"{")
}

fn effective_timestamp(
    context: RequestContext<'_>,
    target: Target,
    member: Option<TimestampFormat>,
) -> Result<Option<TimestampFormat>, EncodeError> {
    match target {
        Target::Shape(index) => Ok(member.or(context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput)?
            .wire()?
            .timestamp_format()?)),
        Target::Prelude(_) => Ok(member),
        _ => Err(EncodeError::Unsupported("unknown timestamp target")),
    }
}

fn shape(context: RequestContext<'_>, target: Target) -> Result<Shape<'_>, EncodeError> {
    match target {
        Target::Shape(index) => context
            .service
            .shape(index)?
            .ok_or(EncodeError::MissingInput),
        Target::Prelude(_) => Err(EncodeError::Unsupported("scalar has no shape metadata")),
        _ => Err(EncodeError::Unsupported("unknown shape target")),
    }
}

fn reserve_tasks(tasks: &mut Vec<Task<'_>>, additional: usize) -> Result<(), EncodeError> {
    if tasks.len().saturating_add(additional) > 65_536 {
        return Err(EncodeError::Limit("frames"));
    }
    tasks
        .try_reserve(additional)
        .map_err(EncodeError::Allocation)
}
