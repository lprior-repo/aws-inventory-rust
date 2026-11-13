//! Owned value tree to canonical bytes.

use std::collections::BTreeMap;

use super::error::RecordError;
use super::value::{FieldId, Time, Value};
use super::varint::{varint_bytes, write_len, write_varint, zigzag};
use super::{MAX_DEPTH, MAX_RECORD_BYTES, tag};

/// Encodes a top-level struct into canonical record bytes.
pub(super) fn encode(value: &Value) -> Result<Vec<u8>, RecordError> {
    let Value::Struct(fields) = value else {
        return Err(RecordError::TopLevelNotStruct);
    };
    let mut out = Vec::new();
    write_struct(&mut out, fields, 1)?;
    within_limit(out.len())?;
    Ok(out)
}

/// Fails once a record of `len` bytes would be over the size limit.
fn within_limit(len: usize) -> Result<(), RecordError> {
    if len > MAX_RECORD_BYTES {
        return Err(RecordError::TooLarge { len });
    }
    Ok(())
}

/// Writes one value; `depth` is the depth a container here would have.
///
/// The limit is checked before every value and before every byte string, so
/// the buffer never grows more than one small scalar past the limit.
fn write_value(out: &mut Vec<u8>, value: &Value, depth: usize) -> Result<(), RecordError> {
    within_limit(out.len())?;
    match value {
        Value::Null => out.push(tag::NULL),
        Value::Bool(false) => out.push(tag::FALSE),
        Value::Bool(true) => out.push(tag::TRUE),
        Value::Int(int) => write_int(out, *int),
        Value::Float(float) => {
            out.push(tag::FLOAT);
            out.extend_from_slice(&float.bits().to_be_bytes());
        }
        Value::Time(time) => write_time(out, *time),
        Value::Text(text) => return write_blob(out, tag::TEXT, text.as_bytes()),
        Value::Bytes(bytes) => return write_blob(out, tag::BYTES, bytes),
        Value::List(items) => return write_list(out, items, depth),
        Value::Map(entries) => return write_map(out, entries, depth),
        Value::Struct(fields) => return write_struct(out, fields, depth),
    }
    Ok(())
}

fn write_int(out: &mut Vec<u8>, int: i64) {
    out.push(tag::INT);
    write_varint(out, zigzag(int));
}

fn write_time(out: &mut Vec<u8>, time: Time) {
    out.push(tag::TIME);
    write_varint(out, zigzag(time.seconds()));
    write_varint(out, u64::from(time.nanos()));
}

fn write_blob(out: &mut Vec<u8>, tag: u8, bytes: &[u8]) -> Result<(), RecordError> {
    out.push(tag);
    write_prefixed(out, bytes)
}

/// Writes a length and then the bytes, unless they would pass the size limit.
fn write_prefixed(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), RecordError> {
    within_limit(out.len().saturating_add(bytes.len()))?;
    write_len(out, bytes.len());
    out.extend_from_slice(bytes);
    Ok(())
}

fn write_list(out: &mut Vec<u8>, items: &[Value], depth: usize) -> Result<(), RecordError> {
    let body = open(out, tag::LIST, items.len(), depth)?;
    for item in items {
        write_value(out, item, depth.saturating_add(1))?;
    }
    close(out, body);
    Ok(())
}

fn write_map(
    out: &mut Vec<u8>,
    entries: &BTreeMap<String, Value>,
    depth: usize,
) -> Result<(), RecordError> {
    let body = open(out, tag::MAP, entries.len(), depth)?;
    for (key, value) in entries {
        write_prefixed(out, key.as_bytes())?;
        write_value(out, value, depth.saturating_add(1))?;
    }
    close(out, body);
    Ok(())
}

fn write_struct(
    out: &mut Vec<u8>,
    fields: &BTreeMap<FieldId, Value>,
    depth: usize,
) -> Result<(), RecordError> {
    let body = open(out, tag::STRUCT, fields.len(), depth)?;
    for (id, value) in fields {
        write_varint(out, u64::from(id.get()));
        write_value(out, value, depth.saturating_add(1))?;
    }
    close(out, body);
    Ok(())
}

/// Starts a container and returns where its body begins.
fn open(out: &mut Vec<u8>, tag: u8, count: usize, depth: usize) -> Result<usize, RecordError> {
    if depth > MAX_DEPTH {
        return Err(RecordError::DepthExceeded);
    }
    out.push(tag);
    let body = out.len();
    write_len(out, count);
    Ok(body)
}

/// Inserts the body length in front of a finished container body.
///
/// The body is written first and shifted once by at most ten bytes, so no
/// temporary buffer is needed and each byte moves at most once per level.
fn close(out: &mut Vec<u8>, body: usize) {
    let len = out.len().saturating_sub(body);
    let (bytes, used) = varint_bytes(u64::try_from(len).unwrap_or(u64::MAX));
    drop(out.splice(body..body, bytes.into_iter().take(used)));
}
