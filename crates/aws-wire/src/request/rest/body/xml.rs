use super::super::super::common::BoundedBody;
use super::super::EncodeError;
use super::super::RequestContext;
use inventory_core::{catalog::{Shape, Target}, record::{ListRef, MapRef, ValueRef}};
mod format;
use format::{binary_namespace, close, element_namespace, empty_namespace, escape_attr, open, open_namespace};
const MAX_ITEMS: usize = 65_536;
#[derive(Clone, Copy)]
enum Task<'a> {
    Value(Target, ValueRef<'a>, &'a str, Option<inventory_core::catalog::TimestampFormat>, Option<inventory_core::catalog::XmlNamespace<'a>>, bool, usize),
    End(&'a str),
    Open(&'a str),
}

pub(super) fn encode(context: RequestContext<'_>, input: Shape<'_>) -> Result<Vec<u8>, EncodeError> {
    let name = match input.wire()?.xml_name()? { Some(name) => name, None => input.name()? };
    let namespace = match input.wire()?.xml_namespace()? { Some(value) => Some(value), None => context.service.wire()?.xml_namespace()? };
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    let mut children = Vec::new();
    for item in input.members()? {
        let member = item?;
        let format = effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let namespace = effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let flattened = member.wire()?.xml_flattened();
        let value = match context.input.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token() => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member missing" }),
                None => continue,
            },
            None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member missing" }),
            None => continue,
        };
        if member.wire()?.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
        if matches!(value, ValueRef::Null) {
            if member.required { return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member is null" }); }
            continue;
        }
        if !matches!(member.wire()?.binding()?, inventory_core::catalog::HttpBinding::Document) { continue }
        if children.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        children.try_reserve(1).map_err(EncodeError::Allocation)?;
        let child_name = match member.wire()?.xml_name()? { Some(name) => name, None => member.name()? };
        children.push((member.target, value, child_name, format, namespace, flattened));
    }
    tasks.try_reserve(children.len().saturating_mul(2).saturating_add(1)).map_err(EncodeError::Allocation)?;
    tasks.push(Task::End(name));
    children.iter().rev().for_each(|(target, value, child, format, namespace, flattened)| tasks.push(Task::Value(*target, *value, child, *format, *namespace, *flattened, 1)));
    open_namespace(&mut body, name, namespace)?;
    write_attributes(context, input, context.input, &mut body, namespace)?;
    body.write(b">")?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}
pub(super) fn encode_payload(context: RequestContext<'_>, target: Target, value: ValueRef<'_>) -> Result<Vec<u8>, EncodeError> {
    let current = shape(context, target)?;
    let name = match current.wire()?.xml_name()? { Some(name) => name, None => current.name()? };
    let namespace = effective_namespace(context, target, None)?;
    let format = effective_timestamp(context, target, None)?;
    let mut body = BoundedBody::new(context.limits.body_bytes());
    let mut tasks = Vec::new();
    write_value(context, &mut body, &mut tasks, target, value, name, format, namespace, false, 0)?;
    write_tasks(context, &mut body, &mut tasks)?;
    Ok(body.into_vec())
}

fn write_tasks(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>) -> Result<(), EncodeError> {
    while let Some(task) = tasks.pop() {
        match task {
            Task::Open(name) => { open(body, name)?; body.write(b">")?; }
            Task::End(name) => close(body, name)?,
            Task::Value(target, value, name, format, namespace, flattened, depth) => write_value(context, body, tasks, target, value, name, format, namespace, flattened, depth)?,
        }
    }
    Ok(())
}

fn write_value(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, value: ValueRef<'_>, name: &str, format: Option<inventory_core::catalog::TimestampFormat>, namespace: Option<inventory_core::catalog::XmlNamespace<'_>>, flattened: bool, depth: usize) -> Result<(), EncodeError> {
    if depth > inventory_core::record::MAX_DEPTH { return Err(EncodeError::Limit("frames")) }
    match value {
        ValueRef::Null => empty_namespace(body, name, namespace),
        ValueRef::Text(text) => element_namespace(body, name, text, namespace),
        ValueRef::Bool(value) => element_namespace(body, name, if value { "true" } else { "false" }, namespace),
        ValueRef::Int(value) => element_namespace(body, name, &value.to_string(), namespace),
        ValueRef::Float(value) => element_namespace(body, name, &value.to_string(), namespace),
        ValueRef::Time(value) => element_namespace(body, name, &super::super::timestamp::format(value, format)?, namespace),
        ValueRef::Bytes(bytes) => binary_namespace(body, name, bytes, namespace),
        ValueRef::List(list) => write_list(context, body, tasks, target, list, name, format, namespace, flattened, depth),
        ValueRef::Map(map) => write_map(context, body, tasks, target, map, name, namespace, flattened, depth),
    }
}

fn write_list(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, list: ListRef<'_>, name: &str, _format: Option<inventory_core::catalog::TimestampFormat>, namespace: Option<inventory_core::catalog::XmlNamespace<'_>>, flattened: bool, depth: usize) -> Result<(), EncodeError> {
    let shape = shape(context, target)?;
    let element_wire = shape.element_wire()?;
    let element_target = shape.element()?.ok_or(EncodeError::Unsupported("list without element"))?;
    if element_wire.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
    let flat = flattened || shape.wire()?.xml_flattened() || element_wire.xml_flattened();
    let item_name = match element_wire.xml_name()? { Some(name) => name, None => "member" };
    let format = effective_timestamp(context, element_target, element_wire.timestamp_format()?)?;
    let item_namespace = effective_namespace(context, element_target, element_wire.xml_namespace()?)?;
    let values = collect_list(list, element_wire.sparse())?;
    tasks.try_reserve(values.len().saturating_mul(2).saturating_add(1)).map_err(EncodeError::Allocation)?;
    if !flat { tasks.push(Task::End(name)); }
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    values.iter().rev().for_each(|value| tasks.push(Task::Value(element_target, *value, item_name, format, item_namespace, false, next)));
    if !flat { open_namespace(body, name, namespace)?; body.write(b">")?; }
    Ok(())
}

fn write_map(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, target: Target, map: MapRef<'_>, name: &str, namespace: Option<inventory_core::catalog::XmlNamespace<'_>>, flattened: bool, depth: usize) -> Result<(), EncodeError> {
    let shape = shape(context, target)?;
    if matches!(shape.kind()?, "structure" | "union") { return write_structure(context, body, tasks, shape, map, name, namespace, depth) }
    let key_name = match shape.map_key_wire()?.xml_name()? { Some(name) => name, None => "key" };
    let value_name = match shape.map_value_wire()?.xml_name()? { Some(name) => name, None => "value" };
    let key_target = shape.map_key()?.ok_or(EncodeError::Unsupported("map without key"))?;
    let value_target = shape.map_value()?.ok_or(EncodeError::Unsupported("map without value"))?;
    if shape.map_key_wire()?.streaming() || shape.map_value_wire()?.streaming() {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let flat = flattened || shape.wire()?.xml_flattened();
    let mut entries = Vec::new();
    for item in map.iter()? {
        if entries.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        let entry = item?;
        if matches!(entry.1, ValueRef::Null) && !shape.map_value_wire()?.sparse() { return Err(EncodeError::InvalidValue { field: None, reason: "null in non-sparse XML map" }); }
        entries.try_reserve(1).map_err(EncodeError::Allocation)?;
        entries.push(entry);
    }
    tasks.try_reserve(entries.len().saturating_mul(4).saturating_add(1)).map_err(EncodeError::Allocation)?;
    if !flat { tasks.push(Task::End(name)); }
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    entries.iter().rev().try_for_each(|(key, value)| {
        let inventory_core::record::KeyRef::Name(text) = key else { return Err(EncodeError::InvalidValue { field: None, reason: "map key is not text" }) };
        let value_format = effective_timestamp(context, value_target, shape.map_value_wire()?.timestamp_format()?)?;
        let key_format = effective_timestamp(context, key_target, shape.map_key_wire()?.timestamp_format()?)?;
        let value_namespace = effective_namespace(context, value_target, shape.map_value_wire()?.xml_namespace()?)?;
        let key_namespace = effective_namespace(context, key_target, shape.map_key_wire()?.xml_namespace()?)?;
        tasks.push(Task::End("entry"));
        tasks.push(Task::Value(value_target, *value, value_name, value_format, value_namespace, false, next));
        tasks.push(Task::Value(key_target, ValueRef::Text(text), key_name, key_format, key_namespace, false, next));
        tasks.push(Task::Open("entry"));
        Ok::<(), EncodeError>(())
    })?;
    if !flat { open_namespace(body, name, namespace)?; body.write(b">")?; }
    Ok(())
}

fn write_structure(context: RequestContext<'_>, body: &mut BoundedBody, tasks: &mut Vec<Task<'_>>, shape: Shape<'_>, map: MapRef<'_>, name: &str, namespace: Option<inventory_core::catalog::XmlNamespace<'_>>, depth: usize) -> Result<(), EncodeError> {
    let mut members = Vec::new();
    for member in shape.members()? {
        if members.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        members.try_reserve(1).map_err(EncodeError::Allocation)?;
        members.push(member?);
    }
    let union = shape.kind()? == "union";
    let mut present = 0_usize;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.try_reserve(members.len().saturating_mul(2).saturating_add(1)).map_err(EncodeError::Allocation)?;
    tasks.push(Task::End(name));
    for member in members.iter().rev() {
        let value = match map.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token() => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member missing" }),
                None => continue,
            },
            None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member missing" }),
            None => continue,
        };
        if member.wire()?.streaming() { return Err(EncodeError::Unsupported("streaming request body")); }
        if matches!(value, ValueRef::Null) {
            if member.required { return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML member is null" }); }
            continue;
        }
        present = present.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
        if member.wire()?.xml_attribute() { continue; }
        let format = effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let child_name = match member.wire()?.xml_name()? { Some(name) => name, None => member.name()? };
        let child_namespace = effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let flattened = member.wire()?.xml_flattened();
        tasks.push(Task::Value(member.target, value, child_name, format, child_namespace, flattened, next));
    }
    if union && present != 1 { return Err(EncodeError::InvalidValue { field: None, reason: "XML union must contain exactly one variant" }); }
    open_namespace(body, name, namespace)?;
    write_attributes(context, shape, map, body, namespace)?;
    body.write(b">")
}

fn write_attributes(context: RequestContext<'_>, shape: Shape<'_>, map: MapRef<'_>, body: &mut BoundedBody, element_namespace: Option<inventory_core::catalog::XmlNamespace<'_>>) -> Result<(), EncodeError> {
    let mut prefixes = Vec::new();
    if let Some(namespace) = element_namespace {
        if let Some(prefix) = namespace.prefix {
            prefixes.try_reserve(1).map_err(EncodeError::Allocation)?;
            prefixes.push((prefix, namespace.uri));
        }
    }
    for item in shape.members()? {
        let member = item?;
        if !member.wire()?.xml_attribute() { continue; }
        let value = match map.get_field(member.id)? {
            Some(value) => value,
            None if member.wire()?.idempotency_token() => match context.idempotency_token {
                Some(token) => ValueRef::Text(token),
                None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML attribute missing" }),
                None => continue,
            },
            None if member.required => return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML attribute missing" }),
            None => continue,
        };
        if matches!(value, ValueRef::Null) {
            if member.required { return Err(EncodeError::InvalidValue { field: Some(member.id), reason: "required XML attribute is null" }); }
            continue;
        }
        let name = match member.wire()?.xml_name()? { Some(name) => name, None => member.name()? };
        let namespace = effective_namespace(context, member.target, member.wire()?.xml_namespace()?)?;
        let prefix = attribute_prefix(body, namespace, &mut prefixes, member.id)?;
        let format = effective_timestamp(context, member.target, member.wire()?.timestamp_format()?)?;
        let value = attribute_value(context, value, member.id, format)?;
        body.write(b" ")?;
        if let Some(prefix) = prefix { body.write(prefix.as_bytes())?; body.write(b":")?; }
        body.write(name.as_bytes())?;
        body.write(b"=\"")?;
        escape_attr(body, &value)?;
        body.write(b"\"")?;
    }
    Ok(())
}

fn attribute_prefix<'a>(body: &mut BoundedBody, namespace: Option<inventory_core::catalog::XmlNamespace<'a>>, prefixes: &mut Vec<(&'a str, &'a str)>, field: inventory_core::record::FieldId) -> Result<Option<&'a str>, EncodeError> {
    let Some(namespace) = namespace else { return Ok(None) };
    let Some(prefix) = namespace.prefix else { return Ok(None) };
    if let Some((_, uri)) = prefixes.iter().find(|(name, _)| *name == prefix) {
        if *uri != namespace.uri { return Err(EncodeError::InvalidValue { field: Some(field), reason: "XML attribute prefix has conflicting namespaces" }); }
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

fn attribute_value(context: RequestContext<'_>, value: ValueRef<'_>, field: inventory_core::record::FieldId, format: Option<inventory_core::catalog::TimestampFormat>) -> Result<String, EncodeError> {
    match value {
        ValueRef::Text(value) => Ok(value.to_owned()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Float(value) => Ok(value.to_string()),
        ValueRef::Time(value) => super::super::timestamp::format(value, format),
        ValueRef::Bytes(value) => {
            let mut body = BoundedBody::new(context.limits.body_bytes());
            super::super::super::body::json::base64(&mut body, value)?;
            String::from_utf8(body.into_vec()).map_err(|_| EncodeError::InvalidValue { field: Some(field), reason: "invalid XML attribute encoding" })
        }
        ValueRef::Null | ValueRef::List(_) | ValueRef::Map(_) => Err(EncodeError::InvalidValue { field: Some(field), reason: "invalid XML attribute value" }),
    }
}

fn collect_list(list: ListRef<'_>, sparse: bool) -> Result<Vec<ValueRef<'_>>, EncodeError> {
    let mut values = Vec::new();
    for item in list.iter()? {
        if values.len() == MAX_ITEMS { return Err(EncodeError::Limit("frames")); }
        let value = item?;
        if matches!(value, ValueRef::Null) && !sparse { return Err(EncodeError::InvalidValue { field: None, reason: "null in non-sparse XML list" }); }
        values.try_reserve(1).map_err(EncodeError::Allocation)?;
        values.push(value);
    }
    Ok(values)
}

fn shape(context: RequestContext<'_>, target: Target) -> Result<Shape<'_>, EncodeError> {
    match target {
        Target::Shape(index) => context.service.shape(index)?.ok_or(EncodeError::MissingInput),
        Target::Prelude(_) => Err(EncodeError::Unsupported("scalar has no shape metadata")),
    }
}
fn effective_timestamp(context: RequestContext<'_>, target: Target, member: Option<inventory_core::catalog::TimestampFormat>) -> Result<Option<inventory_core::catalog::TimestampFormat>, EncodeError> {
    match target {
        Target::Shape(index) => Ok(member.or(context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.wire()?.timestamp_format()?)),
        Target::Prelude(_) => Ok(member),
    }
}

fn effective_namespace<'a>(context: RequestContext<'a>, target: Target, member: Option<inventory_core::catalog::XmlNamespace<'a>>) -> Result<Option<inventory_core::catalog::XmlNamespace<'a>>, EncodeError> {
    match member {
        Some(namespace) => Ok(Some(namespace)),
        None => match target {
            Target::Shape(index) => Ok(context.service.shape(index)?.ok_or(EncodeError::MissingInput)?.wire()?.xml_namespace()?),
            Target::Prelude(_) => Ok(None),
        },
    }
}

