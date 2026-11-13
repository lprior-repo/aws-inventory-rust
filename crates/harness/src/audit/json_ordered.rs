//! Order-preserving JSON for evidence that is rewritten in place.
//!
//! Python's `json` module retains object key order, while `serde_json` sorts
//! keys. Rewriting `catalog/cloudcontrol-capabilities.json` through a sorted
//! map would reorder every object, so the refresh parses into this document
//! model instead and keeps the original order.

use serde_json::{Map, Number, Value};

use crate::HarnessError;

/// JSON value retaining source object order.
#[derive(Clone, Debug)]
pub enum Json {
    /// `null`.
    Null,
    /// Boolean literal.
    Bool(bool),
    /// Signed integer literal.
    Int(i64),
    /// Unsigned integer literal beyond `i64`.
    UInt(u64),
    /// Floating-point literal.
    Float(f64),
    /// String literal.
    String(String),
    /// Ordered array.
    Array(Vec<Json>),
    /// Ordered object entries.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// Parse one complete document.
    pub fn parse(text: &str) -> Result<Self, HarnessError> {
        super::json_parse::parse(text)
    }

    /// Borrow an object member.
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Borrow a nested member.
    pub fn path(&self, keys: &[&str]) -> Option<&Self> {
        let mut current = self;
        for key in keys {
            current = current.get(key)?;
        }
        Some(current)
    }

    /// Mutably borrow an object member.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
        match self {
            Self::Object(entries) => entries
                .iter_mut()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Borrow the ordered entries of an object.
    pub fn as_object(&self) -> Option<&Vec<(String, Self)>> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    /// Mutably borrow the ordered entries of an object.
    pub fn as_object_mut(&mut self) -> Option<&mut Vec<(String, Self)>> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    /// Borrow an array.
    pub fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Mutably borrow an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Borrow a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    /// Borrow a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Borrow an integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            Self::UInt(value) => i64::try_from(*value).ok(),
            _ => None,
        }
    }

    /// Update an object member in place or append it, like `dict.update`.
    pub fn insert(&mut self, key: &str, value: Self) {
        if let Self::Object(entries) = self {
            if let Some((_, slot)) = entries.iter_mut().find(|(name, _)| name == key) {
                *slot = value;
            } else {
                entries.push((key.to_owned(), value));
            }
        }
    }

    /// Python truthiness for containers, numbers and strings.
    pub fn truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(value) => *value,
            Self::Int(value) => *value != 0,
            Self::UInt(value) => *value != 0,
            Self::Float(value) => *value != 0.0,
            Self::String(text) => !text.is_empty(),
            Self::Array(items) => !items.is_empty(),
            Self::Object(entries) => !entries.is_empty(),
        }
    }

    /// Convert into a sorted `serde_json` value for comparisons and printing.
    pub fn to_value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Int(value) => Value::Number(Number::from(*value)),
            Self::UInt(value) => Value::Number(Number::from(*value)),
            Self::Float(value) => Number::from_f64(*value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Self::String(text) => Value::String(text.clone()),
            Self::Array(items) => Value::Array(items.iter().map(Self::to_value).collect()),
            Self::Object(entries) => {
                let mut object = Map::new();
                for (key, value) in entries {
                    object.insert(key.clone(), value.to_value());
                }
                Value::Object(object)
            }
        }
    }

    /// Build from a sorted `serde_json` value.
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(flag) => Self::Bool(*flag),
            Value::Number(number) => {
                if let Some(value) = number.as_i64() {
                    Self::Int(value)
                } else if let Some(value) = number.as_u64() {
                    Self::UInt(value)
                } else if let Some(value) = number.as_f64() {
                    Self::Float(value)
                } else {
                    Self::Null
                }
            }
            Value::String(text) => Self::String(text.clone()),
            Value::Array(items) => Self::Array(items.iter().map(Self::from_value).collect()),
            Value::Object(object) => Self::Object(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::from_value(value)))
                    .collect(),
            ),
        }
    }
}

impl PartialEq for Json {
    fn eq(&self, other: &Self) -> bool {
        py_eq(self, other)
    }
}

/// Python `==` semantics: object order is irrelevant and booleans equal 0/1.
pub fn py_eq(left: &Json, right: &Json) -> bool {
    match (left, right) {
        (Json::Null, Json::Null) => true,
        (Json::Bool(a), Json::Bool(b)) => a == b,
        (Json::String(a), Json::String(b)) => a == b,
        (Json::Int(a), Json::Int(b)) => a == b,
        (Json::UInt(a), Json::UInt(b)) => a == b,
        (Json::Int(a), Json::UInt(b)) => unsigned_eq(*a, *b),
        (Json::UInt(a), Json::Int(b)) => unsigned_eq(*b, *a),
        (Json::Float(a), Json::Float(b)) => a == b,
        (Json::Array(a), Json::Array(b)) => arrays_eq(a, b),
        (Json::Object(a), Json::Object(b)) => objects_eq(a, b),
        (Json::Bool(flag), Json::Int(value)) => *value == i64::from(u8::from(*flag)),
        (Json::Int(value), Json::Bool(flag)) => *value == i64::from(u8::from(*flag)),
        (Json::Bool(flag), Json::UInt(value)) => *value == u64::from(u8::from(*flag)),
        (Json::UInt(value), Json::Bool(flag)) => *value == u64::from(u8::from(*flag)),
        _ => false,
    }
}

fn unsigned_eq(signed: i64, unsigned: u64) -> bool {
    u64::try_from(signed)
        .map(|value| value == unsigned)
        .unwrap_or(false)
}

fn arrays_eq(left: &[Json], right: &[Json]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(x, y)| py_eq(x, y))
}

fn objects_eq(left: &[(String, Json)], right: &[(String, Json)]) -> bool {
    left.len() == right.len()
        && left.iter().all(|(key, value)| {
            right
                .iter()
                .find(|(other, _)| other == key)
                .map(|(_, other)| py_eq(value, other))
                .unwrap_or(false)
        })
}
