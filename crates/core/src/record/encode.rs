//! Fixed-form encoding through a size-bounded writer; minicbor handles all bytes.
use super::{Key, MAX_RECORD_BYTES, RecordError, Time, Value, container_depth};
use minicbor::{Encoder, encode::Write};
use std::collections::BTreeMap;

#[derive(Default)]
struct Bounded(Vec<u8>);

impl Write for Bounded {
    type Error = RecordError;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= MAX_RECORD_BYTES)
            .ok_or(RecordError::SizeLimit)?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

pub(super) fn encode(value: &Value) -> Result<Vec<u8>, RecordError> {
    let mut encoder = Encoder::new(Bounded::default());
    encode_value(&mut encoder, value, 0)?;
    Ok(encoder.into_writer().0)
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
