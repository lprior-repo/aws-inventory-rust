//! Exact bytes, round trips, limits and the canonical scalar types.

use super::super::{Float, MAX_DEPTH, Time};
use super::{Record, RecordError, TestResult, Value, decoded, fields, map, text};

/// A struct holding `lists` nested lists around one null; its depth is `lists + 1`.
fn nested(lists: usize) -> Value {
    let inner = (0..lists).fold(Value::Null, |inner, _| Value::List(vec![inner]));
    fields([(1, inner)])
}

/// The same shape as [`nested`], built byte by byte so any depth can be written.
fn nested_bytes(lists: usize) -> Result<Vec<u8>, std::num::TryFromIntError> {
    let mut value = vec![0u8];
    for _ in 0..lists {
        value = container(8, &[&[1], &value])?;
    }
    container(10, &[&[1, 1], &value])
}

/// A container: tag, body length, body parts. Bodies here stay under 128 bytes.
fn container(tag: u8, parts: &[&[u8]]) -> Result<Vec<u8>, std::num::TryFromIntError> {
    let body = parts.concat();
    let mut bytes = vec![tag, u8::try_from(body.len())?];
    bytes.extend(body);
    Ok(bytes)
}

fn round_trips(value: &Value) -> TestResult {
    let record = Record::encode(value)?;
    let reread = Record::from_bytes(record.as_bytes().to_vec())?;
    assert_eq!(&decoded(&reread), value);
    assert_eq!(reread, record);
    Ok(())
}

#[test]
fn a_small_record_has_exactly_these_bytes() -> TestResult {
    let list = Value::List(vec![Value::Bool(true), Value::Null]);
    let value = fields([(1, Value::Int(-1)), (2, text("hi")), (3, list)]);
    let record = Record::encode(&value)?;
    let expected = [10, 15, 3, 1, 3, 1, 2, 5, 2, 0x68, 0x69, 3, 8, 3, 2, 2, 0];
    assert_eq!(record.as_bytes(), expected);
    Ok(())
}

#[test]
fn an_empty_struct_is_the_smallest_record() -> TestResult {
    let record = Record::encode(&fields([]))?;
    assert_eq!(record.as_bytes(), [10, 1, 0]);
    assert_eq!(decoded(&Record::from_bytes(vec![10, 1, 0])?), fields([]));
    Ok(())
}

#[test]
fn every_kind_of_value_round_trips() -> TestResult {
    let time = Time::new(-1, 999_999_999).ok_or("valid time rejected")?;
    round_trips(&fields([
        (1, Value::Null),
        (2, Value::Bool(false)),
        (3, Value::Bool(true)),
        (4, Value::Float(Float::from(-0.5))),
        (5, text("h\u{e9}llo")),
        (6, Value::Bytes(vec![0, 255])),
        (7, Value::Time(time)),
        (8, Value::List(vec![])),
        (9, map([("k", Value::Int(0))])),
        (10, fields([(7, fields([]))])),
        (u32::MAX, Value::Int(0)),
    ]))
}

#[test]
fn integers_at_every_varint_boundary_round_trip() -> TestResult {
    let ints = [0, -1, 1, 63, 64, -64, -65, 8191, 8192, i64::MAX, i64::MIN];
    round_trips(&fields([(
        1,
        Value::List(ints.into_iter().map(Value::Int).collect()),
    )]))
}

#[test]
fn lengths_that_need_a_second_varint_byte_round_trip() -> TestResult {
    let long_list = Value::List(vec![Value::Int(1); 200]);
    round_trips(&fields([
        (1, text(&"a".repeat(127))),
        (2, text(&"a".repeat(128))),
        (3, long_list),
    ]))
}

#[test]
fn only_a_struct_can_be_the_top_level_value() {
    for value in [Value::Null, Value::List(vec![]), map([])] {
        assert_eq!(Record::encode(&value), Err(RecordError::TopLevelNotStruct));
    }
}

#[test]
fn containers_nest_to_the_depth_limit_and_no_further() -> TestResult {
    let deepest = MAX_DEPTH.saturating_sub(1);
    let record = Record::encode(&nested(deepest))?;
    assert_eq!(record.as_bytes(), nested_bytes(deepest)?);
    assert_eq!(
        decoded(&Record::from_bytes(nested_bytes(deepest)?)?),
        nested(deepest)
    );

    let too_deep = Record::encode(&nested(MAX_DEPTH));
    assert_eq!(too_deep, Err(RecordError::DepthExceeded));
    let too_deep = Record::from_bytes(nested_bytes(MAX_DEPTH)?);
    assert_eq!(too_deep, Err(RecordError::DepthExceeded));
    Ok(())
}

#[test]
fn every_nan_is_one_value_and_signed_zeros_are_two() -> TestResult {
    let payload = f64::from_bits(0x7FF8_0000_0000_0001);
    let nans = [f64::NAN, -f64::NAN, payload].map(Float::from);
    assert!(nans.iter().all(|nan| nan.bits() == 0x7FF8_0000_0000_0000));
    let [first, second] = [f64::NAN, payload].map(|nan| fields([(1, Value::Float(nan.into()))]));
    assert_eq!(Record::encode(&first)?, Record::encode(&second)?);
    assert_ne!(Float::from(0.0), Float::from(-0.0));
    Ok(())
}

#[test]
fn scalar_types_refuse_non_canonical_states() {
    assert_eq!(Time::new(0, 1_000_000_000), None);
    assert_eq!(
        Time::new(0, 999_999_999).map(Time::nanos),
        Some(999_999_999)
    );
    assert_eq!(super::FieldId::new(0), None);
    assert_eq!(super::FieldId::new(7).map(super::FieldId::get), Some(7));
}
