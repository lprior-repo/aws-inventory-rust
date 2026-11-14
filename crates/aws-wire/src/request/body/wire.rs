use super::{wire_scalar::write_key, wire_value::write_value};
use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::{
    catalog::{Service, Target},
    record::{FieldId, KeyRef, ListRef, MapRef, ValueRef},
};

const MAX_PENDING: usize = 1_000_000;
pub(super) const MAX_ITEMS: usize = 1_000_000;
pub(super) const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy)]
pub(super) struct ValueTask<'a> {
    pub(super) target: Target,
    pub(super) value: ValueRef<'a>,
    pub(super) field: Option<FieldId>,
    pub(super) timestamp_format: Option<&'a str>,
    pub(super) media_type: Option<&'a str>,
    pub(super) sparse: bool,
    pub(super) depth: usize,
}

pub(super) enum Task<'a> {
    Value(ValueTask<'a>),
    Key(&'a str),
    Raw(&'static [u8]),
}
pub(super) struct Encoder<'a, 'b> {
    pub(super) service: Service<'a>,
    pub(super) protocol: Protocol,
    pub(super) idempotency_token: Option<&'a str>,
    pub(super) body: &'b mut BoundedBody,
    pub(super) pending: &'b mut Vec<Task<'a>>,
}

pub(super) fn encode(
    service: Service<'_>,
    protocol: Protocol,
    input_shape: Target,
    input: MapRef<'_>,
    idempotency_token: Option<&str>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if matches!(input_shape, Target::Prelude("unit")) {
        return Ok(());
    }
    let mut pending = Vec::new();
    push(&mut pending, root_task(input_shape, input)?)?;
    process_tasks(service, protocol, idempotency_token, body, &mut pending)
}

fn root_task(input_shape: Target, input: MapRef<'_>) -> Result<Task<'_>, EncodeError> {
    Ok(Task::Value(ValueTask {
        target: input_shape,
        value: ValueRef::Map(input),
        field: None,
        timestamp_format: None,
        media_type: None,
        sparse: false,
        depth: 0,
    }))
}

fn process_tasks<'a>(
    service: Service<'a>,
    protocol: Protocol,
    idempotency_token: Option<&'a str>,
    body: &mut BoundedBody,
    pending: &mut Vec<Task<'a>>,
) -> Result<(), EncodeError> {
    while let Some(task) = pending.pop() {
        let mut encoder = Encoder {
            service,
            protocol,
            idempotency_token,
            body,
            pending,
        };
        process_task(&mut encoder, task)?;
    }
    Ok(())
}

fn process_task<'a, 'b>(encoder: &mut Encoder<'a, 'b>, task: Task<'a>) -> Result<(), EncodeError> {
    match task {
        Task::Raw(raw) => encoder.body.write(raw),
        Task::Key(key) => write_key(encoder.body, encoder.protocol, key),
        Task::Value(value) => write_value(encoder, value),
    }
}

pub(super) fn timestamp_format_name(
    format: inventory_core::catalog::TimestampFormat,
) -> &'static str {
    match format {
        inventory_core::catalog::TimestampFormat::DateTime => "date-time",
        inventory_core::catalog::TimestampFormat::HttpDate => "http-date",
        inventory_core::catalog::TimestampFormat::EpochSeconds => "epoch-seconds",
    }
}

pub(super) fn collect_list<'a>(
    list: ListRef<'a>,
    field: Option<FieldId>,
) -> Result<Vec<ValueRef<'a>>, EncodeError> {
    let mut values = Vec::new();
    for value in list.iter()? {
        if values.len() >= MAX_ITEMS {
            return Err(EncodeError::Limit("container items"));
        }
        values.try_reserve(1)?;
        values.push(value?);
    }
    let _ = field;
    Ok(values)
}

pub(super) fn collect_map<'a>(
    map: MapRef<'a>,
    field: Option<FieldId>,
) -> Result<Vec<(KeyRef<'a>, ValueRef<'a>)>, EncodeError> {
    let mut values = Vec::new();
    for entry in map.iter()? {
        if values.len() >= MAX_ITEMS {
            return Err(EncodeError::Limit("container items"));
        }
        values.try_reserve(1)?;
        values.push(entry?);
    }
    let _ = field;
    Ok(values)
}

pub(super) fn push<'a>(pending: &mut Vec<Task<'a>>, task: Task<'a>) -> Result<(), EncodeError> {
    if pending.len() >= MAX_PENDING {
        return Err(EncodeError::Limit("serialization tasks"));
    }
    pending.try_reserve(1)?;
    pending.push(task);
    Ok(())
}

pub(super) fn value_is_null(value: ValueRef<'_>) -> bool {
    matches!(value, ValueRef::Null)
}

pub(super) fn is_text_media_type(value: &str) -> bool {
    value.starts_with("text/") || value.ends_with("+json") || value == "application/json"
}
