use super::{DecodeError, ResponseContext, parser::Key};
use crate::response::timestamp;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use inventory_core::{
    catalog::{Target, TimestampFormat},
    record::{Float, KeyRef, RecordBuilder, Time, ValueRef},
};
use jiter::{Jiter, Peek};

pub(super) fn write(
    parser: &mut Jiter<'_>,
    target: Option<Target>,
    key: Key,
    context: ResponseContext<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    let Some(target) = target else {
        return discard(parser);
    };
    Writer {
        context,
        builder,
        target,
        key,
        kind: target_kind(context, target)?,
    }
    .write(parser)
}

struct Writer<'a, 'b> {
    context: ResponseContext<'a>,
    builder: &'b mut RecordBuilder,
    target: Target,
    key: Key,
    kind: &'static str,
}

impl Writer<'_, '_> {
    fn write(self, parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
        match parser.peek().map_err(syntax)? {
            Peek::Null => {
                parser.next_null().map_err(syntax)?;
                scalar(self.builder, self.key, ValueRef::Null)
            }
            Peek::True | Peek::False => self.boolean(parser),
            Peek::String => self.text(parser.next_str().map_err(syntax)?.as_ref()),
            _ => {
                let number = parser.next_number_bytes().map_err(syntax)?;
                self.number(std::str::from_utf8(number).map_err(syntax)?)
            }
        }
    }

    fn boolean(self, parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
        if !matches!(self.kind, "boolean" | "document") {
            return Err(invalid());
        }
        scalar(
            self.builder,
            self.key,
            ValueRef::Bool(parser.next_bool().map_err(syntax)?),
        )
    }

    fn text(self, text: &str) -> Result<(), DecodeError> {
        match self.kind {
            "blob" => write_blob(text, self.key, self.builder),
            "timestamp" => {
                let time = self.time(text)?;
                scalar(self.builder, self.key, ValueRef::Time(time))
            }
            "float" | "double" => write_nonfinite(text, self.key, self.builder),
            "string" | "enum" | "document" | "bigDecimal" => {
                scalar(self.builder, self.key, ValueRef::Text(text))
            }
            _ => Err(invalid()),
        }
    }

    fn number(self, text: &str) -> Result<(), DecodeError> {
        let value = match self.kind {
            "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" => {
                ValueRef::Int(text.parse().map_err(|_| invalid())?)
            }
            "float" | "double" => ValueRef::Float(Float::new(text.parse().map_err(|_| invalid())?)),
            "bigDecimal" => ValueRef::Text(text),
            "timestamp" => ValueRef::Time(self.time(text)?),
            "document" => document_number(text)?,
            _ => return Err(invalid()),
        };
        scalar(self.builder, self.key, value)
    }

    fn time(&self, text: &str) -> Result<Time, DecodeError> {
        let format = self
            .key
            .timestamp_format()
            .or(timestamp_format(self.context, self.target)?);
        timestamp::parse(text, format)
    }
}

fn document_number(text: &str) -> Result<ValueRef<'_>, DecodeError> {
    match text.parse::<i64>() {
        Ok(value) => Ok(ValueRef::Int(value)),
        Err(_) => Ok(ValueRef::Float(Float::new(
            text.parse().map_err(|_| invalid())?,
        ))),
    }
}
fn write_nonfinite(text: &str, key: Key, builder: &mut RecordBuilder) -> Result<(), DecodeError> {
    let value = match text {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => return Err(invalid()),
    };
    scalar(builder, key, ValueRef::Float(Float::new(value)))
}

fn write_blob(text: &str, key: Key, builder: &mut RecordBuilder) -> Result<(), DecodeError> {
    let capacity = base64::decoded_len_estimate(text.len());
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(capacity)?;
    bytes.resize(capacity, 0);
    let length = STANDARD
        .decode_slice(text, &mut bytes)
        .map_err(|_| invalid())?;
    bytes.truncate(length);
    scalar(builder, key, ValueRef::Bytes(&bytes))
}
pub(super) fn write_raw_payload(
    context: ResponseContext<'_>,
    target: Target,
    key: Key,
    builder: &mut RecordBuilder,
) -> Result<bool, DecodeError> {
    match target_kind(context, target)? {
        "string" | "enum" => {
            let text = std::str::from_utf8(context.body).map_err(syntax)?;
            scalar(builder, key, ValueRef::Text(text))?;
            Ok(true)
        }
        "blob" => {
            scalar(builder, key, ValueRef::Bytes(context.body))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn target_kind(context: ResponseContext<'_>, target: Target) -> Result<&'static str, DecodeError> {
    match target {
        Target::Prelude(name) => Ok(name),
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing shape"))?
            .kind()?),
        _ => Err(DecodeError::Unsupported("non-exhaustive target")),
    }
}

fn timestamp_format(
    context: ResponseContext<'_>,
    target: Target,
) -> Result<Option<TimestampFormat>, DecodeError> {
    match target {
        Target::Prelude(_) => Ok(None),
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing shape"))?
            .wire()?
            .timestamp_format()?),
        _ => Err(DecodeError::Unsupported("non-exhaustive target")),
    }
}

fn scalar(builder: &mut RecordBuilder, key: Key, value: ValueRef<'_>) -> Result<(), DecodeError> {
    match key {
        Key::Ignore => Ok(()),
        Key::Field(id, _, _) => {
            builder.scalar(Some(KeyRef::Field(id)), value)?;
            Ok(())
        }
        Key::Name(name, _) => {
            builder.scalar(Some(KeyRef::Name(&name)), value)?;
            Ok(())
        }
        Key::List(_) => {
            builder.scalar(None, value)?;
            Ok(())
        }
    }
}

impl Key {
    fn timestamp_format(&self) -> Option<TimestampFormat> {
        match self {
            Self::Field(_, format, _) | Self::Name(_, format) | Self::List(format) => *format,
            _ => None,
        }
    }
}

fn discard(parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
    match parser.peek().map_err(syntax)? {
        Peek::Null => parser.next_null().map_err(syntax)?,
        Peek::True | Peek::False => {
            parser.next_bool().map_err(syntax)?;
        }
        Peek::String => {
            parser.next_str().map_err(syntax)?;
        }
        _ => {
            parser.next_number_bytes().map_err(syntax)?;
        }
    }
    Ok(())
}

fn invalid() -> DecodeError {
    DecodeError::Invalid("JSON value does not match modeled shape")
}
fn syntax<E>(_: E) -> DecodeError {
    DecodeError::Invalid("JSON syntax")
}
