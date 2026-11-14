use super::{
    DecodeError,
    index::{Document, Token},
};
use crate::response::timestamp;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use inventory_core::{
    catalog::TimestampFormat,
    record::{Float, RecordBuilder, ValueRef},
};
use std::borrow::Cow;

pub(super) enum Scalar<'a> {
    Text(Cow<'a, str>),
    Bytes(Vec<u8>),
    Bool(bool),
    Int(i64),
    Float(f64),
    Time(inventory_core::record::Time),
    Null,
}

impl Scalar<'_> {
    pub fn write(
        self,
        builder: &mut RecordBuilder,
        key: Option<inventory_core::record::KeyRef<'_>>,
    ) -> Result<(), DecodeError> {
        match self {
            Self::Text(value) => builder.scalar(key, ValueRef::Text(&value))?,
            Self::Bytes(value) => builder.scalar(key, ValueRef::Bytes(&value))?,
            Self::Bool(value) => builder.scalar(key, ValueRef::Bool(value))?,
            Self::Int(value) => builder.scalar(key, ValueRef::Int(value))?,
            Self::Float(value) => builder.scalar(key, ValueRef::Float(Float::new(value)))?,
            Self::Time(value) => builder.scalar(key, ValueRef::Time(value))?,
            Self::Null => builder.scalar(key, ValueRef::Null)?,
        }
        Ok(())
    }
}

pub(super) fn text<'a>(
    document: &'a Document<'_>,
    node: usize,
) -> Result<Cow<'a, str>, DecodeError> {
    let end = document
        .mates
        .get(node)
        .copied()
        .ok_or(DecodeError::Invalid("XML element"))?;
    let start = node
        .checked_add(1)
        .ok_or(DecodeError::Invalid("XML element"))?;
    (start..end).try_fold(Cow::Borrowed(""), |mut value, index| {
        let token = document
            .tokens
            .get(index)
            .ok_or(DecodeError::Invalid("XML token"))?;
        append_token(&mut value, token)?;
        Ok(value)
    })
}

fn append_token<'a>(value: &mut Cow<'a, str>, token: &'a Token<'_>) -> Result<(), DecodeError> {
    match token {
        Token::Text(text) => {
            let decoded = quick_xml::escape::unescape(text.as_ref())
                .map_err(|_| DecodeError::Invalid("XML entity"))?;
            append(value, decoded)
        }
        Token::CData(text) => append(value, Cow::Borrowed(text.as_ref())),
        Token::Reference(character) => {
            let mut buffer = [0_u8; 4];
            extend(value, character.encode_utf8(&mut buffer))
        }
        Token::Start(_) | Token::Empty(_) => Err(DecodeError::Invalid("nested XML scalar")),
        Token::Other => Ok(()),
    }
}

fn append<'a>(value: &mut Cow<'a, str>, text: Cow<'a, str>) -> Result<(), DecodeError> {
    if value.is_empty() {
        *value = text;
        return Ok(());
    }
    extend(value, &text)
}

fn extend(value: &mut Cow<'_, str>, text: &str) -> Result<(), DecodeError> {
    match value {
        Cow::Borrowed(prefix) => *value = Cow::Owned(join(prefix, text)?),
        Cow::Owned(value) => {
            value.try_reserve(text.len())?;
            value.push_str(text);
        }
    }
    Ok(())
}

fn join(prefix: &str, text: &str) -> Result<String, DecodeError> {
    let length = prefix
        .len()
        .checked_add(text.len())
        .ok_or(DecodeError::Limit("XML text"))?;
    let mut value = String::new();
    value.try_reserve_exact(length)?;
    value.push_str(prefix);
    value.push_str(text);
    Ok(value)
}

pub(super) fn parse<'a>(
    kind: &str,
    value: Cow<'a, str>,
    format: Option<TimestampFormat>,
) -> Result<Scalar<'a>, DecodeError> {
    match kind {
        "string" | "enum" => Ok(Scalar::Text(value)),
        "blob" => STANDARD
            .decode(value.trim())
            .map(Scalar::Bytes)
            .map_err(|_| DecodeError::Invalid("invalid XML blob")),
        "boolean" => boolean(value.trim()).map(Scalar::Bool),
        "intEnum" | "byte" | "short" | "integer" | "long" | "bigInteger" => value
            .trim()
            .parse()
            .map(Scalar::Int)
            .map_err(|_| DecodeError::Invalid("invalid XML integer")),
        "float" | "double" => float(value.trim()).map(Scalar::Float),
        "timestamp" => timestamp::parse(value.trim(), format.or(Some(TimestampFormat::DateTime)))
            .map(Scalar::Time),
        "unit" => Ok(Scalar::Null),
        _ => Err(DecodeError::Unsupported("XML scalar kind")),
    }
}

fn boolean(value: &str) -> Result<bool, DecodeError> {
    match value {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(DecodeError::Invalid("invalid XML boolean")),
    }
}

fn float(value: &str) -> Result<f64, DecodeError> {
    match value {
        "NaN" => Ok(f64::NAN),
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        text => text
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .ok_or(DecodeError::Invalid("invalid XML float")),
    }
}
