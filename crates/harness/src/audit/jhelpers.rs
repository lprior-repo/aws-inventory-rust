//! Accessors for the order-preserving JSON used by coverage evidence.

use crate::HarnessError;

use super::json_ordered::Json;

/// Borrow an object member with a concrete reason on absence.
pub(crate) fn field<'a>(value: &'a Json, key: &str) -> Result<&'a Json, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}

/// Borrow a string member.
pub(crate) fn text<'a>(value: &'a Json, key: &str) -> Result<&'a str, HarnessError> {
    field(value, key)?
        .as_str()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

/// Borrow an array member.
pub(crate) fn array<'a>(value: &'a Json, key: &str) -> Result<&'a Vec<Json>, HarnessError> {
    field(value, key)?
        .as_array()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

/// Borrow an object member.
pub(crate) fn object<'a>(
    value: &'a Json,
    key: &str,
) -> Result<&'a Vec<(String, Json)>, HarnessError> {
    field(value, key)?
        .as_object()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

/// Borrow an integer member.
pub(crate) fn integer(value: &Json, key: &str) -> Result<i64, HarnessError> {
    field(value, key)?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}
