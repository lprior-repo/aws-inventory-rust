use super::{Key, Record, RecordError, TestResult, Value, envelope};
use crate::record::MAX_RECORD_BYTES;
use std::collections::BTreeMap;

#[test]
fn individually_valid_children_can_still_exceed_the_aggregate_encoding_budget() -> TestResult {
    Record::encode(&envelope(Value::Bytes(vec![0; 1_048_576])))?;
    let children = (0..17).map(|_| Value::Bytes(vec![0; 1_048_576])).collect();
    let value = envelope(Value::List(children));

    assert!(matches!(
        Record::encode(&value),
        Err(RecordError::SizeLimit)
    ));
    Ok(())
}

#[test]
fn one_oversized_key_returns_the_size_error() {
    let value = Value::Map(BTreeMap::from([(
        Key::Name("k".repeat(MAX_RECORD_BYTES)),
        Value::Null,
    )]));

    assert!(matches!(
        Record::encode(&value),
        Err(RecordError::SizeLimit)
    ));
}

#[test]
fn one_oversized_value_returns_the_size_error() {
    let value = envelope(Value::Bytes(vec![0; MAX_RECORD_BYTES]));

    assert!(matches!(
        Record::encode(&value),
        Err(RecordError::SizeLimit)
    ));
}
