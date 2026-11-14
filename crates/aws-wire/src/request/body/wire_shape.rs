use super::wire::{Encoder, Task, ValueTask, push, value_is_null};
use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::{
    catalog::{Service, Target},
    record::{KeyRef, ValueRef},
};

#[derive(Clone, Copy)]
pub(super) struct ListBinding<'a> {
    pub(super) element: Target,
    pub(super) sparse: bool,
    pub(super) timestamp_format: Option<&'a str>,
    pub(super) media_type: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(super) struct MapBinding<'a> {
    pub(super) key_target: Target,
    pub(super) value_target: Target,
    pub(super) sparse: bool,
    pub(super) timestamp_format: Option<&'a str>,
    pub(super) media_type: Option<&'a str>,
}

struct MapEntry<'a> {
    key: KeyRef<'a>,
    value: ValueRef<'a>,
    index: usize,
    length: usize,
}

#[derive(Clone, Copy)]
struct ListOptions<'a> {
    binding: ListBinding<'a>,
    depth: usize,
}

#[derive(Clone, Copy)]
struct MapOptions<'a> {
    binding: MapBinding<'a>,
    depth: usize,
}

pub(super) fn push_list<'a>(
    encoder: &mut Encoder<'a, '_>,
    values: Vec<ValueRef<'a>>,
    binding: ListBinding<'a>,
    parent: ValueTask<'a>,
) -> Result<(), EncodeError> {
    start_array(encoder.body, encoder.protocol, encoder.pending)?;
    let depth = next_depth(parent)?;
    write_list_values(
        values,
        encoder.protocol,
        parent,
        ListOptions { binding, depth },
        encoder.pending,
    )
}

fn start_array(
    body: &mut BoundedBody,
    protocol: Protocol,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
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
    if value_is_null(value) && !options.binding.sparse {
        return Err(EncodeError::InvalidValue {
            field: parent.field,
            reason: "null in a non-sparse list",
        });
    }
    if index < length.saturating_sub(1) && protocol != Protocol::RpcV2Cbor {
        push(pending, Task::Raw(b","))?;
    }
    push(
        pending,
        Task::Value(ValueTask {
            target: options.binding.element,
            value,
            field: parent.field,
            timestamp_format: options.binding.timestamp_format,
            media_type: options.binding.media_type,
            sparse: options.binding.sparse,
            depth: options.depth,
        }),
    )
}

pub(super) fn push_map<'a>(
    encoder: &mut Encoder<'a, '_>,
    entries: Vec<(KeyRef<'a>, ValueRef<'a>)>,
    binding: MapBinding<'a>,
    parent: ValueTask<'a>,
) -> Result<(), EncodeError> {
    start_map(encoder.body, encoder.protocol, encoder.pending)?;
    let options = MapOptions {
        binding,
        depth: next_depth(parent)?,
    };
    write_map_entries(encoder, entries, parent, options)
}

fn start_map(
    body: &mut BoundedBody,
    protocol: Protocol,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::map_indefinite(body)?;
        return push(pending, Task::Raw(b"\xff"));
    }
    body.write(b"{")?;
    push(pending, Task::Raw(b"}"))
}

fn write_map_entries<'a>(
    encoder: &mut Encoder<'a, '_>,
    entries: Vec<(KeyRef<'a>, ValueRef<'a>)>,
    parent: ValueTask<'a>,
    options: MapOptions<'a>,
) -> Result<(), EncodeError> {
    let length = entries.len();
    for (index, (key, value)) in entries.into_iter().enumerate().rev() {
        queue_map_entry(
            encoder,
            parent,
            options,
            MapEntry {
                key,
                value,
                index,
                length,
            },
        )?;
    }
    Ok(())
}

fn queue_map_entry<'a>(
    encoder: &mut Encoder<'a, '_>,
    parent: ValueTask<'a>,
    options: MapOptions<'a>,
    entry: MapEntry<'a>,
) -> Result<(), EncodeError> {
    check_sparse_null(entry.value, options.binding.sparse, parent.field)?;
    let key = map_key(
        encoder.service,
        entry.key,
        options.binding.key_target,
        parent.field,
    )?;
    queue_map_separator(encoder.protocol, entry.index, entry.length, encoder.pending)?;
    queue_map_value(entry.value, parent, options, encoder.pending)?;
    if encoder.protocol != Protocol::RpcV2Cbor {
        push(encoder.pending, Task::Raw(b":"))?;
    }
    push(encoder.pending, Task::Key(key))
}

fn check_sparse_null(
    value: ValueRef<'_>,
    sparse: bool,
    field: Option<inventory_core::record::FieldId>,
) -> Result<(), EncodeError> {
    if value_is_null(value) && !sparse {
        Err(EncodeError::InvalidValue {
            field,
            reason: "null in a non-sparse map",
        })
    } else {
        Ok(())
    }
}

fn map_key<'a>(
    service: Service<'_>,
    key: KeyRef<'a>,
    target: Target,
    field: Option<inventory_core::record::FieldId>,
) -> Result<&'a str, EncodeError> {
    let is_string = string_target(service, target, field)?;
    match (key, is_string) {
        (KeyRef::Name(name), true) => Ok(name),
        _ => Err(EncodeError::InvalidValue {
            field,
            reason: "map key is not a string",
        }),
    }
}

fn string_target(
    service: Service<'_>,
    target: Target,
    field: Option<inventory_core::record::FieldId>,
) -> Result<bool, EncodeError> {
    match target {
        Target::Prelude(kind) => Ok(kind == "string"),
        Target::Shape(index) => {
            let shape = service.shape(index)?.ok_or(EncodeError::InvalidValue {
                field,
                reason: "map key shape is missing",
            })?;
            Ok(matches!(shape.kind()?, "string" | "enum"))
        }
        _ => Ok(false),
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
    push(
        pending,
        Task::Value(ValueTask {
            target: options.binding.value_target,
            value,
            field: parent.field,
            timestamp_format: options.binding.timestamp_format,
            media_type: options.binding.media_type,
            sparse: options.binding.sparse,
            depth: options.depth,
        }),
    )
}

fn next_depth(parent: ValueTask<'_>) -> Result<usize, EncodeError> {
    parent
        .depth
        .checked_add(1)
        .ok_or(EncodeError::Limit("serialization depth"))
}
