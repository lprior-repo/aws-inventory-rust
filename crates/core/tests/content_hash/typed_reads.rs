//! Typed readers distinguish every supported value kind and preserve read errors.
use inventory_core::record::*;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn values() -> TestResult<[(Value, ValueKind); 9]> {
    Ok([
        (Value::Null, ValueKind::Null),
        (Value::Bool(true), ValueKind::Bool),
        (Value::Int(42), ValueKind::Int),
        (Value::Float(Float::new(1.5)), ValueKind::Float),
        (Value::Text("é".into()), ValueKind::Text),
        (Value::Bytes(vec![0, 0xFF]), ValueKind::Bytes),
        (Value::Time(Time::new(-1, 2)?), ValueKind::Time),
        (Value::List(vec![Value::Int(1)]), ValueKind::List),
        (Value::Map(BTreeMap::new()), ValueKind::Map),
    ])
}

fn record(value: Value) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        value,
    )])))?)
}

fn check_matching_kind((value, kind): (Value, ValueKind)) -> TestResult {
    let encoded = record(value.clone())?;
    let selected = encoded
        .root()
        .get_typed(FieldId::new(1)?, kind)?
        .ok_or("missing value")?;
    assert_eq!(selected.kind(), kind);
    assert_eq!(selected.to_value()?, value);
    Ok(())
}

fn check_other_kinds((value, found): (Value, ValueKind)) -> TestResult {
    let encoded = record(value)?;
    values()?
        .into_iter()
        .filter(|(_, expected)| *expected != found)
        .try_for_each(|(_, expected)| -> TestResult {
            let actual = encoded.root().get_typed(FieldId::new(1)?, expected);
            assert_eq!(
                actual.err(),
                Some(RecordError::KindMismatch {
                    field: FieldId::new(1)?,
                    expected,
                    found,
                })
            );
            Ok(())
        })
}

#[test]
fn every_supported_kind_returns_its_original_value() -> TestResult {
    // Given one independently specified value of each of the nine kinds.
    // When selected with its kind, then kind() and the complete value agree.
    values()?.into_iter().try_for_each(check_matching_kind)
}

#[test]
fn every_nonmatching_kind_returns_a_precise_mismatch() -> TestResult {
    // Given each supported kind and all eight other expectations (72 pairs).
    // When selected, then the exact field, expected kind, and found kind are reported.
    values()?.into_iter().try_for_each(check_other_kinds)
}

#[test]
fn typed_lookup_propagates_unsupported_selected_value_error() -> TestResult {
    // Given structurally valid bytes whose selected value uses forbidden tag 2.
    let borrowed = MapRef::from_bytes(&[0xA1, 1, 0xC2, 0x60])?;
    // When typed lookup runs, then the reader error is not rewritten as a mismatch.
    let actual = borrowed.get_typed(FieldId::new(1)?, ValueKind::Time);
    assert_eq!(actual.err(), Some(RecordError::Tag { tag: 2, offset: 2 }));
    Ok(())
}
