use super::{FieldId, Float, KeyRef, ListRef, MapRef, RecordError, Time, ValueRef};
use minicbor::{Decoder, data::Type};

pub(super) fn definite(count: Option<u64>, offset: usize) -> Result<u64, RecordError> {
    count.ok_or(RecordError::IndefiniteLength { offset })
}

pub(super) fn read_key<'a>(d: &mut Decoder<'a>) -> Result<KeyRef<'a>, RecordError> {
    let offset = d.position();
    match d.datatype()? {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
            let id =
                u32::try_from(d.u64()?).map_err(|_| RecordError::UnsignedOutOfRange { offset })?;
            FieldId::new(id)
                .map(KeyRef::Field)
                .map_err(|_| RecordError::FieldIdZero { offset })
        }
        Type::String => Ok(KeyRef::Name(d.str()?)),
        found => Err(RecordError::KeyType { found, offset }),
    }
}

pub(super) fn read_value<'a>(d: &mut Decoder<'a>) -> Result<ValueRef<'a>, RecordError> {
    match d.datatype()? {
        Type::Null => d.null().map(|()| ValueRef::Null).map_err(Into::into),
        Type::Bool => Ok(ValueRef::Bool(d.bool()?)),
        Type::F64 => Ok(ValueRef::Float(Float::new(d.f64()?))),
        Type::String => Ok(ValueRef::Text(d.str()?)),
        Type::Bytes => Ok(ValueRef::Bytes(d.bytes()?)),
        Type::Tag => read_time(d),
        Type::Array | Type::Map => read_container(d),
        found => read_other(d, found),
    }
}

fn read_other<'a>(d: &mut Decoder<'a>, found: Type) -> Result<ValueRef<'a>, RecordError> {
    match found {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::Int => read_integer(d, found).map(ValueRef::Int),
        _ => Err(unsupported(found, d.position())),
    }
}

fn read_container<'a>(d: &mut Decoder<'a>) -> Result<ValueRef<'a>, RecordError> {
    let is_map = d.datatype()? == Type::Map;
    let offset = d.position();
    let bytes = d
        .input()
        .get(offset..)
        .ok_or(RecordError::Missing { offset })?;
    super::skip::value(d)?;
    Ok(if is_map {
        ValueRef::Map(MapRef { bytes })
    } else {
        ValueRef::List(ListRef { bytes })
    })
}

fn read_time<'a>(d: &mut Decoder<'a>) -> Result<ValueRef<'a>, RecordError> {
    let offset = d.position();
    let tag = d.tag()?.as_u64();
    if tag != 1 {
        return Err(RecordError::Tag { tag, offset });
    }
    let found = d.datatype()?;
    if found == Type::ArrayIndef {
        return Err(RecordError::IndefiniteLength {
            offset: d.position(),
        });
    }
    if found != Type::Array || d.array()? != Some(2) {
        return Err(RecordError::TimeShape { offset });
    }
    read_time_values(d)
}

fn read_time_values<'a>(d: &mut Decoder<'a>) -> Result<ValueRef<'a>, RecordError> {
    let found = d.datatype()?;
    let seconds = read_integer(d, found)?;
    let offset = d.position();
    let nanos = read_nanos(d)?;
    let nanos = u32::try_from(nanos).map_err(|_| RecordError::NanosOutOfRange { nanos, offset })?;
    Ok(ValueRef::Time(Time::new(seconds, nanos)?))
}

fn read_integer(d: &mut Decoder<'_>, found: Type) -> Result<i64, RecordError> {
    let offset = d.position();
    match found {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
            i64::try_from(d.u64()?).map_err(|_| RecordError::UnsignedOutOfRange { offset })
        }
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
            i64::try_from(i128::from(d.int()?))
                .map_err(|_| RecordError::SignedOutOfRange { offset })
        }
        _ => Err(RecordError::ValueType { found, offset }),
    }
}

fn read_nanos(d: &mut Decoder<'_>) -> Result<u64, RecordError> {
    let offset = d.position();
    let nanos = match d.datatype()? {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => d.u64()?,
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
            return Err(RecordError::NegativeNanos {
                nanos: i128::from(d.int()?),
                offset,
            });
        }
        found => return Err(RecordError::ValueType { found, offset }),
    };
    if nanos >= 1_000_000_000 {
        return Err(RecordError::NanosOutOfRange { nanos, offset });
    }
    Ok(nanos)
}

fn unsupported(found: Type, offset: usize) -> RecordError {
    match found {
        Type::F16 => RecordError::FloatType {
            type_name: "f16",
            offset,
        },
        Type::F32 => RecordError::FloatType {
            type_name: "f32",
            offset,
        },
        Type::BytesIndef | Type::StringIndef | Type::ArrayIndef | Type::MapIndef => {
            RecordError::IndefiniteLength { offset }
        }
        _ => RecordError::ValueType { found, offset },
    }
}
