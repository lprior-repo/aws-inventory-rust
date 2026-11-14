use super::super::super::super::common::BoundedBody;
use super::super::super::{EncodeError, RequestContext};
use super::format::{escape_attr, open_namespace};
use super::{MAX_ITEMS, Task, ValueTask, effective_namespace, effective_timestamp};
use inventory_core::{
    catalog::Shape,
    record::{MapRef, ValueRef},
};
pub(super) fn write_structure<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    shape: Shape<'a>,
    map: MapRef<'a>,
    task: ValueTask<'a>,
) -> Result<(), EncodeError> {
    let ValueTask {
        name,
        namespace,
        depth,
        ..
    } = task;
    let mut members = Vec::new();
    for member in shape.members()? {
        if members.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        members.try_reserve(1).map_err(EncodeError::Allocation)?;
        members.push(member?);
    }
    let union = shape.kind()? == "union";
    let mut present = 0_usize;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks
        .try_reserve(members.len().saturating_mul(2).saturating_add(1))
        .map_err(EncodeError::Allocation)?;
    tasks.push(Task::End(name));
    for member in members.iter().rev() {
        let value = match map.get_field(member.id)? {
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
        present = present.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
        if member.wire()?.xml_attribute()? {
            continue;
        }
        let format =
            effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let child_name = match member.wire()?.xml_name()? {
            Some(name) => name,
            None => member.name()?,
        };
        let child_namespace =
            effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let flattened = member.wire()?.xml_flattened()?;
        tasks.push(Task::Value(ValueTask {
            target: member.target,
            value,
            name: child_name,
            format,
            namespace: child_namespace,
            flattened,
            depth: next,
        }));
    }
    if union && present != 1 {
        return Err(EncodeError::InvalidValue {
            field: None,
            reason: "XML union must contain exactly one variant",
        });
    }
    open_namespace(body, name, namespace)?;
    write_attributes(context, shape, map, body, namespace, depth)?;
    body.write(b">")
}

pub(super) fn write_attributes(
    context: RequestContext<'_>,
    shape: Shape<'_>,
    map: MapRef<'_>,
    body: &mut BoundedBody,
    element_namespace: Option<inventory_core::catalog::XmlNamespace<'_>>,
    depth: usize,
) -> Result<(), EncodeError> {
    let mut prefixes = Vec::new();
    if let Some(namespace) = element_namespace
        && let Some(prefix) = namespace.prefix
    {
        prefixes.try_reserve(1).map_err(EncodeError::Allocation)?;
        prefixes.push((prefix, namespace.uri));
    }
    for item in shape.members()? {
        let member = item?;
        if !member.wire()?.xml_attribute()? {
            continue;
        }
        let value = match map.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token()? => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => {
                    return Err(EncodeError::InvalidValue {
                        field: Some(member.id),
                        reason: "required XML attribute missing",
                    });
                }
                None => continue,
            },
            None if depth > 0
                && shape.kind()? != "union"
                && !member.wire()?.client_optional()?
                && let Some(value) = member.wire()?.default_value()?
                && !matches!(value, ValueRef::Null) =>
            {
                value
            }
            None if member.required => {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required XML attribute missing",
                });
            }
            None => continue,
        };
        if matches!(value, ValueRef::Null) {
            if member.required {
                return Err(EncodeError::InvalidValue {
                    field: Some(member.id),
                    reason: "required XML attribute is null",
                });
            }
            continue;
        }
        let name = match member.wire()?.xml_name()? {
            Some(name) => name,
            None => member.name()?,
        };
        let namespace =
            effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let prefix = attribute_prefix(body, namespace, &mut prefixes, member.id)?;
        let format =
            effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let value = attribute_value(context, value, member.id, format)?;
        body.write(b" ")?;
        if let Some(prefix) = prefix {
            body.write(prefix.as_bytes())?;
            body.write(b":")?;
        }
        body.write(name.as_bytes())?;
        body.write(b"=\"")?;
        escape_attr(body, &value)?;
        body.write(b"\"")?;
    }
    Ok(())
}

fn attribute_prefix<'a>(
    body: &mut BoundedBody,
    namespace: Option<inventory_core::catalog::XmlNamespace<'a>>,
    prefixes: &mut Vec<(&'a str, &'a str)>,
    field: inventory_core::record::FieldId,
) -> Result<Option<&'a str>, EncodeError> {
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    let Some(prefix) = namespace.prefix else {
        return Ok(None);
    };
    if let Some((_, uri)) = prefixes.iter().find(|(name, _)| *name == prefix) {
        if *uri != namespace.uri {
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "XML attribute prefix has conflicting namespaces",
            });
        }
        return Ok(Some(prefix));
    }
    prefixes.try_reserve(1).map_err(EncodeError::Allocation)?;
    prefixes.push((prefix, namespace.uri));
    body.write(b" xmlns:")?;
    body.write(prefix.as_bytes())?;
    body.write(b"=\"")?;
    escape_attr(body, namespace.uri)?;
    body.write(b"\"")?;
    Ok(Some(prefix))
}

fn attribute_value(
    context: RequestContext<'_>,
    value: ValueRef<'_>,
    field: inventory_core::record::FieldId,
    format: Option<inventory_core::catalog::TimestampFormat>,
) -> Result<String, EncodeError> {
    match value {
        ValueRef::Text(value) => Ok(value.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_f64().to_string()),
        ValueRef::Time(value) => super::super::super::timestamp::format(value, format),
        ValueRef::Bytes(value) => {
            let mut body = BoundedBody::new(context.limits.body_bytes());
            super::super::super::super::body::json::base64(&mut body, value)?;
            String::from_utf8(body.into_vec()).map_err(|_| EncodeError::InvalidValue {
                field: Some(field),
                reason: "invalid XML attribute encoding",
            })
        }
        ValueRef::Null | ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "invalid XML attribute value",
        }),
    }
}
