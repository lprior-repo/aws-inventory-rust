//! Tests of the record format, split by topic.

use std::collections::BTreeMap;

use super::{FieldId, Record, RecordError, Value};

mod format;
mod properties;
mod reader;
mod reject;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The field id for `id`; an error for zero.
fn fid(id: u32) -> Result<FieldId, &'static str> {
    FieldId::new(id).ok_or("field id zero")
}

/// A struct value from `(id, value)` pairs with non-zero, distinct ids.
fn fields<const N: usize>(entries: [(u32, Value); N]) -> Value {
    let fields: BTreeMap<FieldId, Value> = entries
        .into_iter()
        .filter_map(|(id, value)| Some((FieldId::new(id)?, value)))
        .collect();
    assert_eq!(fields.len(), N, "field ids must be non-zero and distinct");
    Value::Struct(fields)
}

/// A map value from `(key, value)` pairs.
fn map<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

/// A text value.
fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

/// The owned tree a record decodes to, read through the borrowed readers.
fn decoded(record: &Record) -> Value {
    let fields: BTreeMap<FieldId, Value> = record
        .root()
        .iter()
        .map(|(id, value)| (id, value.to_value()))
        .collect();
    Value::Struct(fields)
}

/// The error `from_bytes` gives for `bytes`; a test failure if they are accepted.
fn rejection(bytes: &[u8]) -> Result<RecordError, Box<dyn std::error::Error>> {
    match Record::from_bytes(bytes.to_vec()) {
        Ok(_) => Err("bytes were accepted as a record".into()),
        Err(error) => Ok(error),
    }
}
