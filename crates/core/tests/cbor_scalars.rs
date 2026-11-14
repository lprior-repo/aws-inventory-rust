//! Exact canonical bytes and complete scalar round trips.

mod cbor_support;

use std::collections::BTreeMap;

use cbor_support::{TestResult, envelope, owned};
use inventory_core::record::{FieldId, Float, Key, Record, Time, Value};

const WORKED: [u8; 14] = [
    0xA4, 0x01, 0x20, 0x02, 0x62, 0x68, 0x69, 0x03, 0x82, 0xF5, 0xF6, 0x61, 0x6B, 0x00,
];

fn worked_value() -> TestResult<Value> {
    Ok(Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(-1)),
        (Key::Field(FieldId::new(2)?), Value::Text("hi".into())),
        (
            Key::Field(FieldId::new(3)?),
            Value::List(vec![Value::Bool(true), Value::Null]),
        ),
        (Key::Name("k".into()), Value::Int(0)),
    ])))
}

#[test]
fn encoder_writes_the_worked_example_as_exactly_fourteen_bytes() -> TestResult {
    // Given the contract's mixed-key map.
    let value = worked_value()?;
    // When it is encoded.
    let record = Record::encode(&value)?;
    // Then every byte matches the independent worked example.
    assert_eq!(record.as_bytes(), WORKED);
    assert_eq!(record.into_bytes(), WORKED);
    Ok(())
}

#[test]
fn reader_reconstructs_the_worked_example_from_independent_bytes() -> TestResult {
    // Given the specified bytes, not bytes produced by the encoder.
    let bytes = WORKED.to_vec();
    // When they are validated and read.
    let record = Record::from_bytes(bytes)?;
    // Then the complete tree is reconstructed.
    assert_eq!(owned(&record)?, worked_value()?);
    Ok(())
}

fn scalar_values() -> TestResult<Vec<Value>> {
    Ok(vec![
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::Int(i64::MIN),
        Value::Int(i64::MAX),
        Value::Int(-1),
        Value::Int(0),
        Value::Float(Float::new(-0.0)),
        Value::Float(Float::new(0.0)),
        Value::Float(Float::new(f64::NAN)),
        Value::Float(Float::new(f64::INFINITY)),
        Value::Float(Float::new(f64::NEG_INFINITY)),
        Value::Text("café 🦀".into()),
        Value::Text(String::new()),
        Value::Bytes(vec![0, 0x80, 0xFF]),
        Value::Bytes(Vec::new()),
        Value::Time(Time::new(-1, 999_999_999)?),
        Value::Time(Time::new(i64::MIN, 0)?),
        Value::Time(Time::new(i64::MAX, 999_999_999)?),
        Value::List(Vec::new()),
        Value::Map(BTreeMap::new()),
        envelope(Value::List(vec![Value::Null, Value::Int(7)])),
    ])
}

#[test]
fn one_record_preserves_every_scalar_boundary_and_nested_container() -> TestResult {
    // Given all value kinds together, including the maximum field identifier.
    let values = scalar_values()?;
    let value = Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(u32::MAX)?), Value::List(values)),
        (Key::Name("nested".into()), envelope(Value::Bool(true))),
    ]));
    // When encoded bytes are validated independently and read.
    let record = Record::encode(&value)?;
    let decoded = Record::from_bytes(record.as_bytes().to_vec())?;
    // Then the whole tree and deterministic re-encoding are exact.
    let recovered = owned(&decoded)?;
    assert_eq!(recovered, value);
    assert_eq!(Record::encode(&recovered)?.as_bytes(), record.as_bytes());
    Ok(())
}

#[test]
fn distinct_nan_payloads_write_the_same_canonical_f64_bytes() -> TestResult {
    // Given positive signaling and negative quiet NaNs with different payloads.
    let left = envelope(Value::Float(Float::new(f64::from_bits(
        0x7FF0_0000_0000_0001,
    ))));
    let right = envelope(Value::Float(Float::new(f64::from_bits(
        0xFFF8_0000_0000_1234,
    ))));
    // When encoded.
    let left = Record::encode(&left)?;
    let right = Record::encode(&right)?;
    // Then the entire records agree and the float uses the specified bit pattern.
    assert_eq!(left.as_bytes(), right.as_bytes());
    let expected = [
        0xA1, 0x65, b'v', b'a', b'l', b'u', b'e', 0xFB, 0x7F, 0xF8, 0, 0, 0, 0, 0, 0,
    ];
    assert_eq!(left.as_bytes(), expected);
    Ok(())
}

#[test]
fn signed_zero_retains_its_sign_in_bytes_and_after_reading() -> TestResult {
    // Given the two IEEE signed zeros.
    let positive = envelope(Value::Float(Float::new(0.0)));
    let negative = envelope(Value::Float(Float::new(-0.0)));
    // When encoded and read.
    let positive_record = Record::encode(&positive)?;
    let negative_record = Record::encode(&negative)?;
    // Then both the binary records and the owned values remain distinct.
    assert_ne!(positive_record.as_bytes(), negative_record.as_bytes());
    assert_eq!(owned(&positive_record)?, positive);
    assert_eq!(owned(&negative_record)?, negative);
    assert_ne!(owned(&positive_record)?, owned(&negative_record)?);
    Ok(())
}

#[test]
fn infinities_are_f64_infinities_rather_than_canonical_nan() -> TestResult {
    // Given both infinities with their independently specified IEEE bits.
    [
        (f64::INFINITY, 0x7FF0_0000_0000_0000u64),
        (f64::NEG_INFINITY, 0xFFF0_0000_0000_0000u64),
    ]
    .into_iter()
    .try_for_each(|(input, bits)| -> TestResult {
        // When wrapped, encoded, and read.
        let value = envelope(Value::Float(Float::new(input)));
        let record = Record::encode(&value)?;
        let decoded = Record::from_bytes(record.into_bytes())?;
        // Then construction and reading preserve the infinity bits.
        assert_eq!(Float::new(input).bits(), bits);
        assert_eq!(owned(&decoded)?, value);
        Ok(())
    })
}
