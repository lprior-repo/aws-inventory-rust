use crate::HarnessError;

use super::json_ordered::Json;

pub(crate) fn field<'a>(value: &'a Json, key: &str) -> Result<&'a Json, HarnessError> {
    value
        .get(key)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}

pub(crate) fn text<'a>(value: &'a Json, key: &str) -> Result<&'a str, HarnessError> {
    field(value, key)?
        .as_str()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

pub(crate) fn array<'a>(value: &'a Json, key: &str) -> Result<&'a Vec<Json>, HarnessError> {
    field(value, key)?
        .as_array()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

pub(crate) fn object<'a>(
    value: &'a Json,
    key: &str,
) -> Result<&'a Vec<(String, Json)>, HarnessError> {
    field(value, key)?
        .as_object()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}

pub(crate) fn integer(value: &Json, key: &str) -> Result<i64, HarnessError> {
    field(value, key)?
        .as_i64()
        .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))
}
