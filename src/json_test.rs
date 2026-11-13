//! Test-only JSON accessors that never panic on a missing path.

use serde_json::Value;

/// First array element, or `Null` when the value is not a non-empty array.
pub(crate) fn first(value: &Value) -> &Value {
    nth(value, 0)
}

/// Array element at `index`, or `Null` when absent.
pub(crate) fn nth(value: &Value, index: usize) -> &Value {
    value.get(index).unwrap_or(&Value::Null)
}
