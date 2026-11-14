use super::super::super::super::common::BoundedBody;
use super::super::super::{EncodeError, RequestContext};
use super::format::open_namespace;
use super::{
    MAX_ITEMS, Task, ValueTask, effective_namespace, effective_timestamp, shape, structure,
};
use inventory_core::record::{ListRef, MapRef, ValueRef};

pub(super) fn write_list<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    task: ValueTask<'a>,
    list: ListRef<'a>,
) -> Result<(), EncodeError> {
    let ValueTask {
        target,
        name,
        namespace,
        flattened,
        depth,
        ..
    } = task;
    let shape = shape(context, target)?;
    let element_wire = shape.element_wire()?;
    let element_target = shape
        .element()?
        .ok_or(EncodeError::Unsupported("list without element"))?;
    if element_wire.streaming()? {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let flat = flattened || shape.wire()?.xml_flattened()? || element_wire.xml_flattened()?;
    let item_name = match element_wire.xml_name()? {
        _ if flat => name,
        Some(name) => name,
        None => "member",
    };
    let format = effective_timestamp(context, element_target, element_wire.timestamp_format()?)?;
    let item_namespace =
        effective_namespace(context, element_target, element_wire.xml_namespace()?)?.or(if flat {
            namespace
        } else {
            None
        });
    let values = collect_list(list, element_wire.sparse()?)?;
    tasks
        .try_reserve(values.len().saturating_mul(2).saturating_add(1))
        .map_err(EncodeError::Allocation)?;
    if !flat {
        tasks.push(Task::End(name));
    }
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    values.iter().rev().for_each(|value| {
        tasks.push(Task::Value(ValueTask {
            target: element_target,
            value: *value,
            name: item_name,
            format,
            namespace: item_namespace,
            flattened: false,
            depth: next,
        }))
    });
    if !flat {
        open_namespace(body, name, namespace)?;
        body.write(b">")?;
    }
    Ok(())
}

pub(super) fn write_map<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    task: ValueTask<'a>,
    map: MapRef<'a>,
) -> Result<(), EncodeError> {
    let ValueTask {
        target,
        name,
        namespace,
        flattened,
        depth,
        ..
    } = task;
    let shape = shape(context, target)?;
    if matches!(shape.kind()?, "structure" | "union") {
        return structure::write_structure(context, body, tasks, shape, map, task);
    }
    let key_name = shape.map_key_wire()?.xml_name()?.map_or("key", |name| name);
    let value_name = shape
        .map_value_wire()?
        .xml_name()?
        .map_or("value", |name| name);
    let key_target = shape
        .map_key()?
        .ok_or(EncodeError::Unsupported("map without key"))?;
    let value_target = shape
        .map_value()?
        .ok_or(EncodeError::Unsupported("map without value"))?;
    if shape.map_key_wire()?.streaming()? || shape.map_value_wire()?.streaming()? {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let flat = flattened || shape.wire()?.xml_flattened()?;
    let mut entries = Vec::new();
    for item in map.iter()? {
        if entries.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let entry = item?;
        if matches!(entry.1, ValueRef::Null) && !shape.map_value_wire()?.sparse()? {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "null in non-sparse XML map",
            });
        }
        entries.try_reserve(1).map_err(EncodeError::Allocation)?;
        entries.push(entry);
    }
    tasks
        .try_reserve(entries.len().saturating_mul(4).saturating_add(1))
        .map_err(EncodeError::Allocation)?;
    if !flat {
        tasks.push(Task::End(name));
    }
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    let (entry_name, entry_namespace) = if flat {
        (name, namespace)
    } else {
        ("entry", None)
    };
    entries.iter().rev().try_for_each(|(key, value)| {
        let inventory_core::record::KeyRef::Name(text) = key else {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "map key is not text",
            });
        };
        let value_format = effective_timestamp(
            context,
            value_target,
            shape.map_value_wire()?.timestamp_format()?,
        )?;
        let key_format = effective_timestamp(
            context,
            key_target,
            shape.map_key_wire()?.timestamp_format()?,
        )?;
        let value_namespace = effective_namespace(
            context,
            value_target,
            shape.map_value_wire()?.xml_namespace()?,
        )?;
        let key_namespace =
            effective_namespace(context, key_target, shape.map_key_wire()?.xml_namespace()?)?;
        tasks.push(Task::End(entry_name));
        tasks.push(Task::Value(ValueTask {
            target: value_target,
            value: *value,
            name: value_name,
            format: value_format,
            namespace: value_namespace,
            flattened: false,
            depth: next,
        }));
        tasks.push(Task::Value(ValueTask {
            target: key_target,
            value: ValueRef::Text(text),
            name: key_name,
            format: key_format,
            namespace: key_namespace,
            flattened: false,
            depth: next,
        }));
        tasks.push(Task::Open(entry_name, entry_namespace));
        Ok::<(), EncodeError>(())
    })?;
    if !flat {
        open_namespace(body, name, namespace)?;
        body.write(b">")?;
    }
    Ok(())
}
pub(super) fn collect_list(
    list: ListRef<'_>,
    sparse: bool,
) -> Result<Vec<ValueRef<'_>>, EncodeError> {
    let mut values = Vec::new();
    for item in list.iter()? {
        if values.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let value = item?;
        if matches!(value, ValueRef::Null) && !sparse {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "null in non-sparse XML list",
            });
        }
        values.try_reserve(1).map_err(EncodeError::Allocation)?;
        values.push(value);
    }
    Ok(values)
}
