use super::{BuildError, RecordBuilder};
use crate::record::{
    FieldId, Float, Key, KeyRef, MAX_DEPTH, MAX_RECORD_BYTES, Record, Time, Value, ValueRef,
};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn nested_stream_matches_canonical_encoding_for_mixed_values() -> TestResult {
    let field = FieldId::new(8)?;
    let inner = BTreeMap::from([
        (
            Key::Name("alpha".to_owned()),
            Value::Float(Float::new(f64::NAN)),
        ),
        (
            Key::Field(FieldId::new(2)?),
            Value::Time(Time::new(-12, 345_678_901)?),
        ),
    ]);
    let values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Int(i64::MIN),
        Value::Float(Float::new(f64::INFINITY)),
        Value::Float(Float::new(f64::NEG_INFINITY)),
        Value::Text("value".to_owned()),
        Value::Bytes(vec![0, 1, 255]),
        Value::Map(inner),
    ];
    let expected = Value::Map(BTreeMap::from([
        (Key::Field(field), Value::List(values)),
        (Key::Name("empty".to_owned()), Value::Map(BTreeMap::new())),
    ]));
    let expected_record = Record::encode(&expected)?;
    let mut builder = builder(4096, 32, MAX_DEPTH)?;
    builder.begin_map(None)?;
    builder.begin_list(Some(KeyRef::Field(field)))?;
    builder.scalar(None, ValueRef::Null)?;
    builder.scalar(None, ValueRef::Bool(true))?;
    builder.scalar(None, ValueRef::Int(i64::MIN))?;
    builder.scalar(None, ValueRef::Float(Float::new(f64::INFINITY)))?;
    builder.scalar(None, ValueRef::Float(Float::new(f64::NEG_INFINITY)))?;
    builder.scalar(None, ValueRef::Text("value"))?;
    builder.scalar(None, ValueRef::Bytes(&[0, 1, 255]))?;
    builder.begin_map(None)?;
    builder.scalar(
        Some(KeyRef::Name("alpha")),
        ValueRef::Float(Float::new(f64::NAN)),
    )?;
    builder.scalar(
        Some(KeyRef::Field(FieldId::new(2)?)),
        ValueRef::Time(Time::new(-12, 345_678_901)?),
    )?;
    builder.end()?;
    builder.end()?;
    builder.begin_map(Some(KeyRef::Name("empty")))?;
    builder.end()?;
    builder.end()?;
    assert_eq!(builder.finish()?.as_bytes(), expected_record.as_bytes());
    Ok(())
}

#[test]
fn mixed_map_keys_are_sorted_by_record_key_order() -> TestResult {
    let field = FieldId::new(1)?;
    let expected = Value::Map(BTreeMap::from([
        (Key::Field(field), Value::Int(1)),
        (Key::Name("a".to_owned()), Value::Int(2)),
    ]));
    let expected_record = Record::encode(&expected)?;
    let mut builder = builder(32, 3, 1)?;
    builder.begin_map(None)?;
    builder.scalar(Some(KeyRef::Name("a")), ValueRef::Int(2))?;
    builder.scalar(Some(KeyRef::Field(field)), ValueRef::Int(1))?;
    builder.end()?;
    assert_eq!(builder.finish()?.as_bytes(), expected_record.as_bytes());
    Ok(())
}

#[test]
fn builder_rejects_invalid_contexts_and_stays_terminal() -> TestResult {
    let mut root = builder(32, 8, 4)?;
    assert!(root.begin_list(None).is_err());
    assert!(root.begin_map(None).is_err());
    assert!(root.finish().is_err());

    let mut map = builder(32, 8, 4)?;
    map.begin_map(None)?;
    assert!(map.scalar(None, ValueRef::Null).is_err());
    assert!(map.end().is_err());
    assert!(map.finish().is_err());

    let mut duplicate = builder(32, 8, 4)?;
    duplicate.begin_map(None)?;
    duplicate.scalar(Some(KeyRef::Name("x")), ValueRef::Null)?;
    assert!(
        duplicate
            .scalar(Some(KeyRef::Name("x")), ValueRef::Bool(false))
            .is_err()
    );
    assert!(duplicate.finish().is_err());
    Ok(())
}

#[test]
fn second_root_and_unmatched_or_unfinished_containers_are_rejected() -> TestResult {
    let mut second = builder(32, 4, 2)?;
    second.begin_map(None)?;
    second.end()?;
    assert!(second.begin_map(None).is_err());
    assert!(second.finish().is_err());

    let mut unmatched = builder(32, 4, 2)?;
    assert!(unmatched.end().is_err());
    assert!(unmatched.finish().is_err());

    let mut unfinished = builder(32, 4, 2)?;
    unfinished.begin_map(None)?;
    assert!(matches!(unfinished.finish(), Err(BuildError::State(_))));
    Ok(())
}

#[test]
fn limits_are_inclusive_and_failures_are_terminal() -> TestResult {
    let mut exact = builder(1, 1, 1)?;
    exact.begin_map(None)?;
    exact.end()?;
    assert_eq!(exact.finish()?.as_bytes(), &[0xa0]);

    let mut bytes = builder(2, 2, 1)?;
    bytes.begin_map(None)?;
    bytes.scalar(Some(KeyRef::Name("x")), ValueRef::Null)?;
    assert!(bytes.end().is_err());
    assert!(bytes.finish().is_err());
    let mut nodes = builder(16, 1, 1)?;
    nodes.begin_map(None)?;
    assert!(
        nodes
            .scalar(Some(KeyRef::Field(FieldId::new(1)?)), ValueRef::Null)
            .is_err()
    );
    assert!(nodes.finish().is_err());

    let mut depth = builder(16, 4, 1)?;
    depth.begin_map(None)?;
    assert!(
        depth
            .begin_map(Some(KeyRef::Field(FieldId::new(1)?)))
            .is_err()
    );
    assert!(depth.finish().is_err());

    assert!(RecordBuilder::new(0, 1, 1).is_err());
    assert!(RecordBuilder::new(MAX_RECORD_BYTES.saturating_add(1), 1, 1).is_err());
    assert!(RecordBuilder::new(1, 0, 1).is_err());
    assert!(RecordBuilder::new(1, 65_537, 1).is_err());
    assert!(RecordBuilder::new(1, 1, 0).is_err());
    assert!(RecordBuilder::new(1, 1, MAX_DEPTH.saturating_add(1)).is_err());
    Ok(())
}

fn builder(bytes: usize, nodes: usize, depth: usize) -> Result<RecordBuilder, BuildError> {
    RecordBuilder::new(bytes, nodes, depth)
}
