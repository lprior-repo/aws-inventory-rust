use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::{record::{KeyRef, MapRef, ValueRef}, catalog::Target};
use super::wire::{Task, ValueTask, push, value_is_null};

struct ListOptions<'a> {
    element: Target,
    sparse: bool,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
    depth: usize,
}

struct MapOptions<'a> {
    key_target: Target,
    value_target: Target,
    sparse: bool,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
    depth: usize,
}

pub(super) fn push_list<'a>(
    body: &mut BoundedBody,
    protocol: Protocol,
    values: Vec<ValueRef<'a>>,
    element: Target,
    sparse: bool,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
    parent: ValueTask<'a>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    start_array(body, protocol, pending)?;
    let depth = next_depth(parent)?;
    write_list_values(values, protocol, parent, ListOptions { element, sparse, timestamp_format, media_type, depth }, pending)
}

fn start_array(body: &mut BoundedBody, protocol: Protocol, pending: &mut Vec<Task<'_>>) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::array_indefinite(body)?;
        return push(pending, Task::Raw(b"\xff"));
    }
    body.write(b"[")?;
    push(pending, Task::Raw(b"]"))
}

fn write_list_values<'a>(
    values: Vec<ValueRef<'a>>,
    protocol: Protocol,
    parent: ValueTask<'a>,
    options: ListOptions<'a>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    let length = values.len();
    for (index, value) in values.into_iter().enumerate().rev() {
        queue_list_value(value, protocol, parent, options, index, length, pending)?;
    }
    Ok(())
}

fn queue_list_value<'a>(
    value: ValueRef<'a>,
    protocol: Protocol,
    parent: ValueTask<'a>,
    options: ListOptions<'a>,
    index: usize,
    length: usize,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    if value_is_null(value) && !options.sparse {
        return Err(EncodeError::InvalidValue { field: parent.field, reason: "null in a non-sparse list" });
    }
    if index < length.saturating_sub(1) && protocol != Protocol::RpcV2Cbor {
        push(pending, Task::Raw(b","))?;
    }
    push(pending, Task::Value(ValueTask {
        target: options.element,
        value,
        field: parent.field,
        timestamp_format: options.timestamp_format,
        media_type: options.media_type,
        sparse: options.sparse,
        depth: options.depth,
    }))
}

pub(super) fn push_map<'a>(
    body: &mut BoundedBody,
    protocol: Protocol,
    entries: Vec<(KeyRef<'a>, ValueRef<'a>)>,
    key_target: Target,
    value_target: Target,
    sparse: bool,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
    parent: ValueTask<'a>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    start_map(body, protocol, pending)?;
    let depth = next_depth(parent)?;
    let options = MapOptions { key_target, value_target, sparse, timestamp_format, media_type, depth };
    write_map_entries(entries, protocol, parent, options, pending)
}

fn start_map(body: &mut BoundedBody, protocol: Protocol, pending: &mut Vec<Task<'_>>) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::map_indefinite(body)?;
        return push(pending, Task::Raw(b"\xff"));
    }
    body.write(b"{")?;
    push(pending, Task::Raw(b"}"))
}

fn write_map_entries<'a>(
    entries: Vec<(KeyRef<'a>, ValueRef<'a>)>,
    protocol: Protocol,
    parent: ValueTask<'a>,
    options: MapOptions<'a>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    let length = entries.len();
    for (index, (key, value)) in entries.into_iter().enumerate().rev() {
        queue_map_entry(key, value, protocol, parent, options, index, length, pending)?;
    }
    Ok(())
}

fn queue_map_entry<'a>(
    key: KeyRef<'a>,
    value: ValueRef<'a>,
    protocol: Protocol,
    parent: ValueTask<'a>,
    options: MapOptions<'a>,
    index: usize,
    length: usize,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    check_sparse_null(value, options.sparse, parent.field)?;
    let key = map_key(key, options.key_target, parent.field)?;
    queue_map_separator(protocol, index, length, pending)?;
    queue_map_value(value, parent, options, pending)?;
    if protocol != Protocol::RpcV2Cbor {
        push(pending, Task::Raw(b":"))?;
    }
    push(pending, Task::Key(key))
}

fn check_sparse_null(value: ValueRef<'_>, sparse: bool, field: Option<inventory_core::record::FieldId>) -> Result<(), EncodeError> {
    if value_is_null(value) && !sparse {
        Err(EncodeError::InvalidValue { field, reason: "null in a non-sparse map" })
    } else {
        Ok(())
    }
}

fn map_key<'a>(key: KeyRef<'a>, target: Target, field: Option<inventory_core::record::FieldId>) -> Result<&'a str, EncodeError> {
    match key {
        KeyRef::Name(name) if target == Target::Prelude("string") => Ok(name),
        _ => Err(EncodeError::InvalidValue { field, reason: "map key is not a string" }),
    }
}

fn queue_map_separator(
    protocol: Protocol,
    index: usize,
    length: usize,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    if index < length.saturating_sub(1) && protocol != Protocol::RpcV2Cbor {
        push(pending, Task::Raw(b","))?;
    }
    Ok(())
}

fn queue_map_value<'a>(
    value: ValueRef<'a>,
    parent: ValueTask<'a>,
    options: MapOptions<'a>,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    push(pending, Task::Value(ValueTask {
        target: options.value_target,
        value,
        field: parent.field,
        timestamp_format: options.timestamp_format,
        media_type: options.media_type,
        sparse: options.sparse,
        depth: options.depth,
    }))
}

fn next_depth(parent: ValueTask<'_>) -> Result<usize, EncodeError> {
    parent.depth.checked_add(1).ok_or(EncodeError::Limit("serialization depth"))
}
