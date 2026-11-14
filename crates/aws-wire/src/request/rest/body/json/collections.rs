use super::super::super::super::RequestContext;
use super::super::super::super::common::BoundedBody;
use super::super::super::EncodeError;
use super::{MAX_ITEMS, Task, effective_timestamp, reserve_tasks, shape};
use inventory_core::{
    catalog::Target,
    record::{ListRef, MapRef, ValueRef},
};
pub(super) fn write_list<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    target: Target,
    list: ListRef<'a>,
    depth: usize,
) -> Result<(), EncodeError> {
    let current = shape(context, target)?;
    let (element, sparse, format) = match current.kind()? {
        "document" => (target, true, None),
        "list" => {
            let element = current
                .element()?
                .ok_or(EncodeError::Unsupported("list without element"))?;
            let wire = current.element_wire()?;
            if wire.streaming()? {
                return Err(EncodeError::Unsupported("streaming request body"));
            }
            (
                element,
                current.wire()?.sparse()?,
                effective_timestamp(context, element, wire.timestamp_format()?)?,
            )
        }
        _ => {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "value does not match modeled shape",
            });
        }
    };
    let mut values = Vec::new();
    for value in list.iter()? {
        if values.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let value = value?;
        if matches!(value, ValueRef::Null) && !sparse {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "null in non-sparse list",
            });
        }
        values.try_reserve(1).map_err(EncodeError::Allocation)?;
        values.push(value);
    }
    reserve_tasks(tasks, values.len().saturating_mul(2).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"]"));
    values
        .iter()
        .rev()
        .enumerate()
        .try_for_each(|(offset, value)| {
            if offset > 0 {
                tasks.push(Task::Raw(b","));
            }
            tasks.push(Task::Value(element, *value, None, format, next));
            Ok::<(), EncodeError>(())
        })?;
    body.write(b"[")
}

pub(super) fn write_map<'a>(
    context: RequestContext<'a>,
    body: &mut BoundedBody,
    tasks: &mut Vec<Task<'a>>,
    target: Target,
    map: MapRef<'a>,
    depth: usize,
) -> Result<(), EncodeError> {
    if matches!(target, Target::Prelude("unit")) {
        return write_unit(body, map);
    }
    let current = shape(context, target)?;
    if matches!(current.kind()?, "structure" | "union") {
        return super::write_structure(context, body, tasks, current, map, depth);
    }
    let (value_target, sparse, format, streaming) = match current.kind()? {
        "document" => (target, true, None, false),
        "map" => {
            let value_target = current
                .map_value()?
                .ok_or(EncodeError::Unsupported("map without value"))?;
            let value_wire = current.map_value_wire()?;
            (
                value_target,
                current.wire()?.sparse()?,
                effective_timestamp(context, value_target, value_wire.timestamp_format()?)?,
                current.map_key_wire()?.streaming()? || value_wire.streaming()?,
            )
        }
        _ => {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "value does not match modeled shape",
            });
        }
    };
    if streaming {
        return Err(EncodeError::Unsupported("streaming request body"));
    }
    let mut entries = Vec::new();
    for entry in map.iter()? {
        if entries.len() == MAX_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let entry = entry?;
        if matches!(entry.1, ValueRef::Null) && !sparse {
            return Err(EncodeError::InvalidValue {
                field: None,
                reason: "null in non-sparse map",
            });
        }
        entries.try_reserve(1).map_err(EncodeError::Allocation)?;
        entries.push(entry);
    }
    reserve_tasks(tasks, entries.len().saturating_mul(4).saturating_add(1))?;
    let next = depth.checked_add(1).ok_or(EncodeError::Limit("frames"))?;
    tasks.push(Task::Raw(b"}"));
    entries
        .iter()
        .rev()
        .enumerate()
        .try_for_each(|(offset, (key, value))| {
            let inventory_core::record::KeyRef::Name(name) = key else {
                return Err(EncodeError::InvalidValue {
                    field: None,
                    reason: "map key is not text",
                });
            };
            if offset > 0 {
                tasks.push(Task::Raw(b","));
            }
            tasks.push(Task::Value(value_target, *value, None, format, next));
            tasks.push(Task::Raw(b":"));
            tasks.push(Task::Name(name));
            Ok(())
        })?;
    body.write(b"{")
}

fn write_unit(body: &mut BoundedBody, map: MapRef<'_>) -> Result<(), EncodeError> {
    if map.iter()?.next().is_some() {
        return Err(EncodeError::InvalidValue {
            field: None,
            reason: "unit value has members",
        });
    }
    body.write(b"{}")
}
