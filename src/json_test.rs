use serde_json::Value;

pub(crate) fn first(value: &Value) -> &Value {
    nth(value, 0)
}

pub(crate) fn nth(value: &Value, index: usize) -> &Value {
    value.get(index).unwrap_or(&Value::Null)
}
