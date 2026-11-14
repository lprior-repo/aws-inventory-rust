use super::super::DecodeError;
use inventory_core::{
    catalog::Target,
    record::{Float, KeyRef, RecordBuilder, Time, ValueRef},
};
use minicbor::{Decoder, data::Type};

pub(super) fn scalar(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    target: Option<Target>,
    key: Option<KeyRef<'_>>,
    kind: Option<&str>,
) -> Result<(), DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::String | Type::StringIndef => return text_scalar(decoder, builder, key, kind),
        Type::Bytes | Type::BytesIndef => return bytes_scalar(decoder, builder, key, kind),
        _ => {}
    }
    let value = read_value(decoder)?;
    validate_scalar(kind, value)?;
    if matches!(kind, Some("structure" | "union")) && matches!(value, ValueRef::Null) {
        return Ok(());
    }
    if target.is_some() {
        builder.scalar(key, value)?;
    }
    Ok(())
}

fn read_value(decoder: &mut Decoder<'_>) -> Result<ValueRef<'static>, DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::Null => {
            decoder.null().map_err(|_| invalid_cbor())?;
            Ok(ValueRef::Null)
        }
        Type::Undefined => {
            decoder.undefined().map_err(|_| invalid_cbor())?;
            Ok(ValueRef::Null)
        }
        Type::Bool => decoder
            .bool()
            .map(ValueRef::Bool)
            .map_err(|_| invalid_cbor()),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => unsigned(decoder).map(ValueRef::Int),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
            decoder.i64().map(ValueRef::Int).map_err(|_| invalid_cbor())
        }
        Type::F16 | Type::F32 | Type::F64 => {
            floating(decoder).map(|value| ValueRef::Float(Float::new(value)))
        }
        _ => Err(DecodeError::Invalid("unsupported CBOR scalar")),
    }
}

fn unsigned(decoder: &mut Decoder<'_>) -> Result<i64, DecodeError> {
    i64::try_from(decoder.u64().map_err(|_| invalid_cbor())?)
        .map_err(|_| DecodeError::Invalid("integer outside record range"))
}

fn floating(decoder: &mut Decoder<'_>) -> Result<f64, DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::F16 => decoder.f16().map(f64::from).map_err(|_| invalid_cbor()),
        Type::F32 => decoder.f32().map(f64::from).map_err(|_| invalid_cbor()),
        Type::F64 => decoder.f64().map_err(|_| invalid_cbor()),
        _ => Err(DecodeError::Invalid("unsupported CBOR float")),
    }
}

fn text_scalar(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
    kind: Option<&str>,
) -> Result<(), DecodeError> {
    if !matches!(kind, Some("string" | "enum" | "intEnum" | "document")) {
        return Err(DecodeError::Invalid(
            "CBOR text does not match modeled shape",
        ));
    }
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::String => {
            builder.scalar(
                key,
                ValueRef::Text(decoder.str().map_err(|_| invalid_cbor())?),
            )?;
        }
        Type::StringIndef => text_indefinite(decoder, builder, key)?,
        _ => return Err(DecodeError::Invalid("expected CBOR text")),
    }
    Ok(())
}

fn text_indefinite(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
) -> Result<(), DecodeError> {
    let mut text = String::new();
    for chunk in decoder.str_iter().map_err(|_| invalid_cbor())? {
        let chunk = chunk.map_err(|_| invalid_cbor())?;
        text.try_reserve(chunk.len())?;
        text.push_str(chunk);
    }
    builder.scalar(key, ValueRef::Text(&text))?;
    Ok(())
}

fn bytes_scalar(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
    kind: Option<&str>,
) -> Result<(), DecodeError> {
    if !matches!(kind, Some("blob" | "document")) {
        return Err(DecodeError::Invalid(
            "CBOR bytes do not match modeled shape",
        ));
    }
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::Bytes => {
            builder.scalar(
                key,
                ValueRef::Bytes(decoder.bytes().map_err(|_| invalid_cbor())?),
            )?;
        }
        Type::BytesIndef => bytes_indefinite(decoder, builder, key)?,
        _ => return Err(DecodeError::Invalid("expected CBOR bytes")),
    }
    Ok(())
}

fn bytes_indefinite(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
) -> Result<(), DecodeError> {
    let mut bytes = Vec::new();
    for chunk in decoder.bytes_iter().map_err(|_| invalid_cbor())? {
        let chunk = chunk.map_err(|_| invalid_cbor())?;
        bytes.try_reserve(chunk.len())?;
        bytes.extend_from_slice(chunk);
    }
    builder.scalar(key, ValueRef::Bytes(&bytes))?;
    Ok(())
}

pub(super) struct ScalarContext<'a> {
    pub(super) target: Option<Target>,
    pub(super) key: Option<KeyRef<'a>>,
    pub(super) kind: Option<&'a str>,
}

pub(super) fn scalar_tag(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    context: ScalarContext<'_>,
) -> Result<(), DecodeError> {
    let tag = decoder.tag().map_err(|_| invalid_cbor())?.as_u64();
    if tag != 1 || context.kind != Some("timestamp") {
        return Err(DecodeError::Unsupported("CBOR tag for modeled value"));
    }
    let time = timestamp_value(decoder)?;
    if context.target.is_some() {
        builder.scalar(context.key, ValueRef::Time(time))?;
    }
    Ok(())
}

fn timestamp_value(decoder: &mut Decoder<'_>) -> Result<Time, DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => timestamp_unsigned(decoder),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
            Time::new(decoder.i64().map_err(|_| invalid_cbor())?, 0).map_err(Into::into)
        }
        Type::F16 => crate::response::timestamp::from_float(f64::from(
            decoder.f16().map_err(|_| invalid_cbor())?,
        )),
        Type::F32 => crate::response::timestamp::from_float(f64::from(
            decoder.f32().map_err(|_| invalid_cbor())?,
        )),
        Type::F64 => {
            crate::response::timestamp::from_float(decoder.f64().map_err(|_| invalid_cbor())?)
        }
        _ => Err(DecodeError::Invalid("invalid CBOR timestamp")),
    }
}

fn timestamp_unsigned(decoder: &mut Decoder<'_>) -> Result<Time, DecodeError> {
    let seconds = i64::try_from(decoder.u64().map_err(|_| invalid_cbor())?)
        .map_err(|_| DecodeError::Invalid("timestamp range"))?;
    Time::new(seconds, 0).map_err(Into::into)
}

fn validate_scalar(kind: Option<&str>, value: ValueRef<'_>) -> Result<(), DecodeError> {
    let valid = matches!(
        (kind, value),
        (None, _)
            | (_, ValueRef::Null)
            | (Some("document"), _)
            | (Some("boolean"), ValueRef::Bool(_))
            | (Some("blob"), ValueRef::Bytes(_))
            | (Some("string" | "enum"), ValueRef::Text(_))
            | (
                Some("intEnum" | "byte" | "short" | "integer" | "long" | "bigInteger"),
                ValueRef::Int(_)
            )
            | (Some("float" | "double" | "bigDecimal"), ValueRef::Float(_))
    );
    if valid {
        Ok(())
    } else {
        Err(DecodeError::Invalid(
            "CBOR scalar does not match modeled shape",
        ))
    }
}

pub(super) fn skip_scalar(decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::String | Type::StringIndef => decoder
            .str_iter()
            .map_err(|_| invalid_cbor())?
            .try_for_each(|part| part.map(|_| ()).map_err(|_| invalid_cbor())),
        Type::Bytes | Type::BytesIndef => decoder
            .bytes_iter()
            .map_err(|_| invalid_cbor())?
            .try_for_each(|part| part.map(|_| ()).map_err(|_| invalid_cbor())),
        Type::Break => Err(DecodeError::Invalid("unexpected CBOR break")),
        _ => skip_primitive(decoder),
    }
}

fn skip_primitive(decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::Null => decoder.null().map_err(|_| invalid_cbor()),
        Type::Undefined => decoder.undefined().map_err(|_| invalid_cbor()),
        Type::Bool => decoder.bool().map(|_| ()).map_err(|_| invalid_cbor()),
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
            decoder.u64().map(|_| ()).map_err(|_| invalid_cbor())
        }
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
            decoder.i64().map(|_| ()).map_err(|_| invalid_cbor())
        }
        Type::F16 | Type::F32 | Type::F64 => floating(decoder).map(|_| ()),
        Type::Simple => decoder.simple().map(|_| ()).map_err(|_| invalid_cbor()),
        _ => Err(DecodeError::Invalid("unexpected CBOR item")),
    }
}

pub(super) fn invalid_cbor() -> DecodeError {
    DecodeError::Invalid("malformed CBOR")
}
