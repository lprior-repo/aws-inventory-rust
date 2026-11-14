use super::{Key, Record, RecordError, Time, Value};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn envelope(value: Value) -> Value {
    Value::Map(BTreeMap::from([(Key::Name("value".into()), value)]))
}

mod encoding;
mod structure;
mod time;
mod views;
