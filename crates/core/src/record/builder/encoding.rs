use super::{BuildError, Entry, Frame};
use crate::record::{Key, Time, ValueRef};
use minicbor::{Encoder, encode::Write};

struct Bounded {
    bytes: Vec<u8>,
    limit: usize,
    aggregate_limit: usize,
    existing: usize,
}

impl Bounded {
    fn new(limit: usize, aggregate_limit: usize, existing: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            aggregate_limit,
            existing,
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for Bounded {
    type Error = BuildError;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let next = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|size| *size <= self.limit)
            .ok_or(BuildError::Limit("bytes"))?;
        self.existing
            .checked_add(next)
            .filter(|size| *size <= self.aggregate_limit)
            .ok_or(BuildError::Limit("buffer"))?;
        self.bytes.try_reserve(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

pub(super) fn scalar(
    value: ValueRef<'_>,
    max_bytes: usize,
    buffer_limit: usize,
    buffered: usize,
) -> Result<Vec<u8>, BuildError> {
    let mut encoder = Encoder::new(Bounded::new(max_bytes, buffer_limit, buffered));
    match value {
        ValueRef::Null => encode(encoder.null())?,
        ValueRef::Bool(item) => encode(encoder.bool(item))?,
        ValueRef::Int(item) => encode(encoder.i64(item))?,
        ValueRef::Float(item) => encode(encoder.f64(item.to_f64()))?,
        ValueRef::Text(item) => encode(encoder.str(item))?,
        ValueRef::Bytes(item) => encode(encoder.bytes(item))?,
        ValueRef::Time(item) => write_time(&mut encoder, item)?,
        ValueRef::Map(_) | ValueRef::List(_) => {
            return Err(BuildError::State("container passed as scalar"));
        }
    }
    Ok(encoder.into_writer().finish())
}

pub(super) fn container(
    frame: &Frame,
    max_bytes: usize,
    aggregate_limit: usize,
    existing: usize,
) -> Result<Vec<u8>, BuildError> {
    let mut encoder = Encoder::new(Bounded::new(max_bytes, aggregate_limit, existing));
    let count = u64::try_from(frame.entries.len()).map_err(|_| BuildError::Limit("nodes"))?;
    if frame.map {
        encode(encoder.map(count))?;
        write_entries(&mut encoder, &frame.entries)?;
    } else {
        encode(encoder.array(count))?;
        write_values(&mut encoder, &frame.entries)?;
    }
    Ok(encoder.into_writer().finish())
}

fn write_entries(encoder: &mut Encoder<Bounded>, entries: &[Entry]) -> Result<(), BuildError> {
    for entry in entries {
        match entry.key.as_ref() {
            Some(Key::Field(field)) => encode(encoder.u32(field.value()))?,
            Some(Key::Name(name)) => encode(encoder.str(name))?,
            None => return Err(BuildError::State("map entry has no key")),
        }
        raw(encoder, &entry.bytes)?;
    }
    Ok(())
}

fn write_values(encoder: &mut Encoder<Bounded>, entries: &[Entry]) -> Result<(), BuildError> {
    for entry in entries {
        if entry.key.is_some() {
            return Err(BuildError::State("list entry has a key"));
        }
        raw(encoder, &entry.bytes)?;
    }
    Ok(())
}

fn raw(encoder: &mut Encoder<Bounded>, bytes: &[u8]) -> Result<(), BuildError> {
    encoder.writer_mut().write_all(bytes)
}

fn write_time(encoder: &mut Encoder<Bounded>, time: Time) -> Result<(), BuildError> {
    encode(encoder.tag(minicbor::data::Tag::new(1)))?;
    encode(encoder.array(2))?;
    encode(encoder.i64(time.seconds()))?;
    encode(encoder.u32(time.nanoseconds()))?;
    Ok(())
}

fn encode<T>(result: Result<T, minicbor::encode::Error<BuildError>>) -> Result<(), BuildError> {
    result.map(|_| ()).map_err(|error| {
        if let Some(source) = error.into_write() {
            return source;
        }
        BuildError::Record(crate::record::RecordError::Encode)
    })
}
pub(super) fn header_size(length: usize) -> usize {
    match length {
        0..=23 => 1,
        24..=255 => 2,
        256..=65_535 => 3,
        _ => 5,
    }
}

pub(super) fn unsigned_size(value: u32) -> usize {
    match value {
        0..=23 => 1,
        24..=255 => 2,
        256..=65_535 => 3,
        _ => 5,
    }
}
