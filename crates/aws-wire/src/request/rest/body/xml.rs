use super::super::super::common::BoundedBody;
use super::super::EncodeError;
use super::super::RequestContext;
use inventory_core::{
    catalog::{Shape, Target},
    record::ValueRef,
};
mod format;
use format::{binary_namespace, close, element_namespace, empty_namespace, open_namespace};
pub(super) const MAX_ITEMS: usize = 65_536;
mod collections;
mod structure;
#[derive(Clone, Copy)]
pub(super) struct ValueTask<'a> {
    target: Target,
    value: ValueRef<'a>,
    name: &'a str,
    format: Option<inventory_core::catalog::TimestampFormat>,
    namespace: Option<inventory_core::catalog::XmlNamespace<'a>>,
    flattened: bool,
    depth: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Task<'a> {
    Value(ValueTask<'a>),
    End(&'a str),
    Open(&'a str, Option<inventory_core::catalog::XmlNamespace<'a>>),
}

pub(super) fn encode<'a>(
    context: RequestContext<'a>,
    input: Shape<'a>,
) -> Result<Vec<u8>, EncodeError> {
    let name = match input.wire()?.xml_name()? {
        Some(name) => name,
        None => input.name()?,
    };
    let namespace = match input.wire()?.xml_namespace()? {
        Some(value) => Some(value),
        None => context.service.wire()?.xml_namespace()?,
    };
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    let mut children = Vec::new();
    for item in input.members()? {
        let member = item?;
        let format =
            effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let namespace =
            effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let flattened = member.wire()?.xml_flattened()?;
        let value = match context.input.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token()? => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => {
                    return Err(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "required XML member missing",
                    });
                }
                None => continue,
            },
            None if member.required => {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required XML member missing",
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
                    reason: "required XML member is null",
                });
            }
            continue;
        }
        if !matches!(
            member.wire()?.binding()?,
            inventory_core::catalog::HttpBinding::Document
        ) {
            continue;
        }
        if member.wire()?.xml_attribute()? {
            continue;
        }
        if children.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        children.try_reserve(1).map_err(EncodeError::Allocation)?;
        let child_name = match member.wire()?.xml_name()? {
            Some(name) => name,
            None => member.name()?,
        };
        children.push((
            member.target,
            value,
            child_name,
            format,
            namespace,
            flattened,
        ));
    }
    tasks
        .try_reserve(children.len().saturating_mul(2).saturating_add(1))
        .map_err(EncodeError::Allocation)?;
    tasks.push(Task::End(name));
    children
        .iter()
        .rev()
        .for_each(|(target, value, child, format, namespace, flattened)| {
            tasks.push(Task::Value(ValueTask {
                target: *target,
                value: *value,
                name: child,
                format: *format,
                namespace: *namespace,
                flattened: *flattened,
                depth: 1,
            }))
        });
    open_namespace(&mut body, name, namespace)?;
    structure::write_attributes(context, input, context.input, &mut body, namespace, 0)?;
    body.write(b">")?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}
pub(super) fn encode_payload<'a>(
    context: RequestContext<'a>,
    target: Target,
    value: ValueRef<'a>,
    member_name: Option<&'a str>,
) -> Result<Vec<u8>, EncodeError> {
    let current = shape(context, target)?;
    let name = match (member_name, current.wire()?.xml_name()?) {
        (Some(name), _) | (None, Some(name)) => name,
        (None, None) => current.name()?,
    };
    let namespace = effective_namespace(context, target, None)?;
    let format = effective_timestamp(context, target, None)?;
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    let task = ValueTask {
        target,
        value,
        name,
        format,
        namespace,
        flattened: false,
        depth: 0,
    };
    write_value(context, &mut body, &mut tasks, task)?;
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
            Task::Open(name, namespace) => {
                open_namespace(body, name, namespace)?;
                body.write(b">")?;
            }
            Task::End(name) => close(body, name)?,
            Task::Value(task) => write_value(context, body, tasks, task)?,
        }
    }
    Ok(())
}

fn write_value<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    task: ValueTask<'a>,
) -> Result<(), EncodeError> {
    let ValueTask {
        value,
        name,
        format,
        namespace,
        depth,
        ..
    } = task;
    if depth > inventory_core::record::MAX_DEPTH {
        return Err(EncodeError::Limit("frames"));
    }
    match value {
        ValueRef::Null => empty_namespace(body, name, namespace),
        ValueRef::Text(text) => element_namespace(body, name, text, namespace),
        ValueRef::Bool(value) => {
            element_namespace(body, name, if value { "true" } else { "false" }, namespace)
        }
        ValueRef::Int(value) => element_namespace(body, name, &value.to_string(), namespace),
        ValueRef::Float(value) => {
            element_namespace(body, name, &value.to_f64().to_string(), namespace)
        }
        ValueRef::Time(value) => element_namespace(
            body,
            name,
            &super::super::timestamp::format(value, format)?,
            namespace,
        ),
        ValueRef::Bytes(bytes) => binary_namespace(body, name, bytes, namespace),
        ValueRef::List(list) => collections::write_list(context, body, tasks, task, list),
        ValueRef::Map(map) => collections::write_map(context, body, tasks, task, map),
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
fn effective_timestamp(
    context: RequestContext<'_>,
    target: Target,
    member: Option<inventory_core::catalog::TimestampFormat>,
) -> Result<Option<inventory_core::catalog::TimestampFormat>, EncodeError> {
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

fn effective_namespace<'a>(
    context: RequestContext<'a>,
    target: Target,
    member: Option<inventory_core::catalog::XmlNamespace<'a>>,
) -> Result<Option<inventory_core::catalog::XmlNamespace<'a>>, EncodeError> {
    match member {
        Some(namespace) => Ok(Some(namespace)),
        None => match target {
            Target::Shape(index) => Ok(context
                .service
                .shape(index)?
                .ok_or(EncodeError::MissingInput)?
                .wire()?
                .xml_namespace()?),
            Target::Prelude(_) => Ok(None),
            _ => Err(EncodeError::Unsupported("unknown namespace target")),
        },
    }
}
