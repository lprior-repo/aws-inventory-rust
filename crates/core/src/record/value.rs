//! Domain types for inventory records.

use std::collections::BTreeMap;

use super::error::RecordError;

use super::containers::{ListRef, MapRef};

/// A non-zero field identifier used as a map key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(u32);

impl FieldId {
    /// Construct a nonzero identifier.
    /// # Errors
    /// Returns `FieldIdZero` for zero.
    pub fn new(value: u32) -> Result<Self, RecordError> {
        if value == 0 {
            return Err(RecordError::FieldIdZero { offset: 0 });
        }
        Ok(Self(value))
    }

    /// Return the identifier.
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// A floating-point value with canonicalized NaN.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Float(u64);

impl Float {
    /// Canonicalize NaNs while preserving infinities and signed zeros.
    pub fn new(value: f64) -> Self {
        Self(if value.is_nan() {
            0x7FF8_0000_0000_0000
        } else {
            value.to_bits()
        })
    }

    /// Return the canonical IEEE 754 bits.
    pub fn bits(&self) -> u64 {
        self.0
    }

    /// Recover the floating-point value.
    pub fn to_f64(&self) -> f64 {
        f64::from_bits(self.0)
    }
}

/// A time value: seconds and nanoseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    seconds: i64,
    nanoseconds: u32,
}

impl Time {
    /// Construct seconds and fractional nanoseconds.
    /// # Errors
    /// Returns `NanosOutOfRange` for a fraction of one second or more.
    pub fn new(seconds: i64, nanoseconds: u32) -> Result<Self, RecordError> {
        if nanoseconds >= 1_000_000_000 {
            return Err(RecordError::NanosOutOfRange {
                nanos: u64::from(nanoseconds),
                offset: 0,
            });
        }
        Ok(Self {
            seconds,
            nanoseconds,
        })
    }

    /// Return whole seconds, which may be negative.
    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Return the fractional nanoseconds.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }
}

/// A map key: either a field ID or a name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    /// Numeric field identifier, ordered before every name.
    Field(FieldId),
    /// UTF-8 name, ordered by its bytes.
    Name(String),
}

/// Borrowed form of a map key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyRef<'a> {
    /// Borrowed numeric identifier.
    Field(FieldId),
    /// Borrowed UTF-8 name.
    Name(&'a str),
}

impl KeyRef<'_> {
    /// Copy this key into an owned key.
    pub fn to_key(self) -> Key {
        match self {
            KeyRef::Field(f) => Key::Field(f),
            KeyRef::Name(s) => Key::Name(s.to_string()),
        }
    }
}

/// An owned record value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// CBOR null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed 64-bit integer.
    Int(i64),
    /// Canonical 64-bit float.
    Float(Float),
    /// UTF-8 text.
    Text(String),
    /// Uninterpreted bytes.
    Bytes(Vec<u8>),
    /// Timestamp.
    Time(Time),
    /// Ordered sequence.
    List(Vec<Value>),
    /// Deterministically ordered entries.
    Map(BTreeMap<Key, Value>),
}

/// Borrowed view of a record value.
#[derive(Clone, Copy, Debug)]
pub enum ValueRef<'a> {
    /// CBOR null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed 64-bit integer.
    Int(i64),
    /// Canonical 64-bit float.
    Float(Float),
    /// Borrowed UTF-8 text.
    Text(&'a str),
    /// Borrowed uninterpreted bytes.
    Bytes(&'a [u8]),
    /// Timestamp.
    Time(Time),
    /// Borrowed sequence.
    List(ListRef<'a>),
    /// Borrowed map.
    Map(MapRef<'a>),
}

impl ValueRef<'_> {
    /// Rebuild an owned value, refusing more than 32 nested containers.
    /// # Errors
    /// Returns a read error for unsupported forms or `DepthLimit` past 32 containers.
    pub fn to_value(&self) -> Result<Value, RecordError> {
        self.to_value_at_depth(0)
    }

    fn to_value_at_depth(self, depth: usize) -> Result<Value, RecordError> {
        match self {
            Self::Null => Ok(Value::Null),
            Self::Bool(value) => Ok(Value::Bool(value)),
            Self::Int(value) => Ok(Value::Int(value)),
            Self::Float(value) => Ok(Value::Float(value)),
            Self::Text(value) => Ok(Value::Text(value.to_string())),
            Self::Bytes(value) => Ok(Value::Bytes(value.to_vec())),
            Self::Time(value) => super::container_depth(depth).map(|_| Value::Time(value)),
            Self::List(list) => Self::owned_list(list, super::container_depth(depth)?),
            Self::Map(map) => Self::owned_map(map, super::container_depth(depth)?),
        }
    }

    fn owned_list(list: ListRef<'_>, depth: usize) -> Result<Value, RecordError> {
        list.iter()?
            .map(|entry| entry?.to_value_at_depth(depth))
            .collect::<Result<_, _>>()
            .map(Value::List)
    }

    fn owned_map(map: MapRef<'_>, depth: usize) -> Result<Value, RecordError> {
        map.iter()?
            .map(|entry| {
                let (key, value) = entry?;
                Ok((key.to_key(), value.to_value_at_depth(depth)?))
            })
            .collect::<Result<_, _>>()
            .map(Value::Map)
    }
}
