use super::{Key, MAX_RECORD_BYTES, RecordError, Time, Value, container_depth};
use minicbor::{Encoder, encode::Write};
use std::collections::BTreeMap;

struct Bounded {
    bytes: Vec<u8>,
    limit: usize,
}

impl Bounded {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}

impl Write for Bounded {
    type Error = RecordError;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= self.limit)
            .ok_or(RecordError::SizeLimit)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

pub(super) fn encode(value: &Value, max_bytes: usize) -> Result<Vec<u8>, RecordError> {
    let mut encoder = Encoder::new(Bounded::new(max_bytes));
    encode_value(&mut encoder, value, 0)?;
    Ok(encoder.into_writer().bytes)
}

pub(super) fn encode_list_record(items: &[Value]) -> Result<Vec<u8>, RecordError> {
    let mut encoder = Encoder::new(Bounded::new(MAX_RECORD_BYTES));
    encode_list_value(&mut encoder, items, 0)?;
    Ok(encoder.into_writer().bytes)
}
fn encode_list_value(
    e: &mut Encoder<Bounded>,
    items: &[Value],
    depth: usize,
) -> Result<(), RecordError> {
    encode_list(e, items, container_depth(depth)?)
}
fn encode_value(e: &mut Encoder<Bounded>, value: &Value, depth: usize) -> Result<(), RecordError> {
    match value {
        Value::Null => e.null(),
        Value::Bool(v) => e.bool(*v),
        Value::Int(v) => e.i64(*v),
        Value::Float(v) => e.f64(v.to_f64()),
        Value::Text(v) => e.str(v),
        Value::Bytes(v) => e.bytes(v),
        Value::Time(v) => return encode_time(e, v, depth),
        Value::List(v) => return encode_list(e, v, container_depth(depth)?),
        Value::Map(v) => return encode_map(e, v, container_depth(depth)?),
    }?;
    Ok(())
}

fn encode_list(e: &mut Encoder<Bounded>, items: &[Value], depth: usize) -> Result<(), RecordError> {
    e.array(u64::try_from(items.len()).map_err(|_| RecordError::SizeLimit)?)?;
    items
        .iter()
        .try_for_each(|item| encode_value(e, item, depth))
}

fn encode_map(
    e: &mut Encoder<Bounded>,
    entries: &BTreeMap<Key, Value>,
    depth: usize,
) -> Result<(), RecordError> {
    e.map(u64::try_from(entries.len()).map_err(|_| RecordError::SizeLimit)?)?;
    entries.iter().try_for_each(|(key, value)| {
        match key {
            Key::Field(id) => {
                e.u32(id.value())?;
            }
            Key::Name(name) => {
                e.str(name)?;
            }
        }
        encode_value(e, value, depth)
    })
}

fn encode_time(e: &mut Encoder<Bounded>, time: &Time, depth: usize) -> Result<(), RecordError> {
    container_depth(depth)?;
    e.tag(minicbor::data::Tag::new(1))?
        .array(2)?
        .i64(time.seconds())?
        .u32(time.nanoseconds())?;
    Ok(())
}

#[cfg(test)]
mod tests;
