use std::collections::BTreeMap;

use super::error::RecordError;

use super::containers::{ListRef, MapRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(u32);

impl FieldId {
    pub fn new(value: u32) -> Result<Self, RecordError> {
        if value == 0 {
            return Err(RecordError::FieldIdZero { offset: 0 });
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Float(u64);

impl Float {
    pub fn new(value: f64) -> Self {
        Self(if value.is_nan() {
            0x7FF8_0000_0000_0000
        } else {
            value.to_bits()
        })
    }

    pub fn bits(&self) -> u64 {
        self.0
    }

    pub fn to_f64(&self) -> f64 {
        f64::from_bits(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    seconds: i64,
    nanoseconds: u32,
}

impl Time {
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

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Field(FieldId),

    Name(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyRef<'a> {
    Field(FieldId),

    Name(&'a str),
}

impl KeyRef<'_> {
    pub fn to_key(self) -> Key {
        match self {
            KeyRef::Field(f) => Key::Field(f),
            KeyRef::Name(s) => Key::Name(s.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,

    Bool(bool),

    Int(i64),

    Float(Float),

    Text(String),

    Bytes(Vec<u8>),

    Time(Time),

    List(Vec<Value>),

    Map(BTreeMap<Key, Value>),
}

#[derive(Clone, Copy, Debug)]
pub enum ValueRef<'a> {
    Null,

    Bool(bool),

    Int(i64),

    Float(Float),

    Text(&'a str),

    Bytes(&'a [u8]),

    Time(Time),

    List(ListRef<'a>),

    Map(MapRef<'a>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Null,

    Bool,

    Int,

    Float,

    Text,

    Bytes,

    Time,

    List,

    Map,
}

impl ValueRef<'_> {
    pub fn to_value(&self) -> Result<Value, RecordError> {
        self.to_value_at_depth(0)
    }

    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Null => ValueKind::Null,
            Self::Bool(_) => ValueKind::Bool,
            Self::Int(_) => ValueKind::Int,
            Self::Float(_) => ValueKind::Float,
            Self::Text(_) => ValueKind::Text,
            Self::Bytes(_) => ValueKind::Bytes,
            Self::Time(_) => ValueKind::Time,
            Self::List(_) => ValueKind::List,
            Self::Map(_) => ValueKind::Map,
        }
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
