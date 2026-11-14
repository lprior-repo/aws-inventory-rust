use std::collections::BTreeMap;

use inventory_core::record::{Key, Record, RecordError, Value, ValueRef};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn envelope(value: Value) -> Value {
    Value::Map(BTreeMap::from([(Key::Name("value".to_string()), value)]))
}

pub fn owned(record: &Record) -> Result<Value, RecordError> {
    ValueRef::Map(record.root()).to_value()
}
