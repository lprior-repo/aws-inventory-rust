//! Borrowed view of one value inside a validated record.

use super::containers::{ListRef, MapRef, StructRef};
use super::cursor::Cursor;
use super::tag;
use super::value::{Float, Time, Value};

/// A value read in place. Scalars are copied out; text, bytes and containers
/// borrow from the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRef<'a> {
    /// An explicit null.
    Null,
    /// A boolean.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// A float.
    Float(Float),
    /// Text.
    Text(&'a str),
    /// A blob.
    Bytes(&'a [u8]),
    /// A timestamp.
    Time(Time),
    /// A list, not yet read.
    List(ListRef<'a>),
    /// A map, not yet read.
    Map(MapRef<'a>),
    /// A structure, not yet read.
    Struct(StructRef<'a>),
}

impl ValueRef<'_> {
    /// Rebuilds the owned value tree.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match *self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(value),
            Self::Int(value) => Value::Int(value),
            Self::Float(value) => Value::Float(value),
            Self::Text(value) => Value::Text(value.to_owned()),
            Self::Bytes(value) => Value::Bytes(value.to_vec()),
            Self::Time(value) => Value::Time(value),
            Self::List(list) => Value::List(list.iter().map(|item| item.to_value()).collect()),
            Self::Map(map) => Value::Map(
                map.iter()
                    .map(|(key, value)| (key.to_owned(), value.to_value()))
                    .collect(),
            ),
            Self::Struct(fields) => Value::Struct(
                fields
                    .iter()
                    .map(|(id, value)| (id, value.to_value()))
                    .collect(),
            ),
        }
    }
}

/// Reads the value at the cursor. Containers are bounded, not parsed.
///
/// Returns `None` only on malformed input, which a validated record never has.
pub(super) fn read_value<'a>(cursor: &mut Cursor<'a>) -> Option<ValueRef<'a>> {
    Some(match cursor.byte().ok()? {
        tag::NULL => ValueRef::Null,
        tag::FALSE => ValueRef::Bool(false),
        tag::TRUE => ValueRef::Bool(true),
        tag::INT => ValueRef::Int(cursor.zigzag().ok()?),
        tag::FLOAT => ValueRef::Float(cursor.float().ok()?),
        tag::TEXT => ValueRef::Text(cursor.text().ok()?),
        tag::BYTES => ValueRef::Bytes(cursor.blob().ok()?),
        tag::TIME => ValueRef::Time(cursor.time().ok()?),
        tag::LIST => ValueRef::List(ListRef::new(cursor.blob().ok()?)),
        tag::MAP => ValueRef::Map(MapRef::new(cursor.blob().ok()?)),
        tag::STRUCT => ValueRef::Struct(StructRef::new(cursor.blob().ok()?)),
        _ => return None,
    })
}

/// Steps over the value at the cursor without decoding it.
///
/// Text is not checked for UTF-8 and containers are skipped by their length.
pub(super) fn skip_value(cursor: &mut Cursor<'_>) -> Option<()> {
    match cursor.byte().ok()? {
        tag::NULL | tag::FALSE | tag::TRUE => Some(()),
        tag::INT => cursor.varint().ok().map(drop),
        tag::FLOAT => cursor.take(8).ok().map(drop),
        tag::TIME => {
            cursor.varint().ok()?;
            cursor.varint().ok().map(drop)
        }
        tag::TEXT | tag::BYTES | tag::LIST | tag::MAP | tag::STRUCT => cursor.blob().ok().map(drop),
        _ => None,
    }
}
