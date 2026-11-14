use inventory_core::record::*;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn wrap_in_map(value: Value) -> Value {
    Value::Map(BTreeMap::from([(Key::Name("value".into()), value)]))
}
fn owned(record: &Record) -> Result<Value, RecordError> {
    ValueRef::Map(record.root()).to_value()
}
fn decoded(value: &Value) -> Result<Value, RecordError> {
    Record::encode(&wrap_in_map(value.clone()))?
        .root()
        .get_name("value")?
        .ok_or(RecordError::Missing { offset: 0 })?
        .to_value()
}
fn scalar_values() -> Vec<Value> {
    vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(0),
        Value::Int(42),
        Value::Int(-42),
        Value::Int(i64::MAX),
        Value::Int(i64::MIN),
        Value::Text("hello".into()),
        Value::Text("".into()),
        Value::Text("café".into()),
        Value::Text("🚀".into()),
        Value::Bytes(vec![0, 1, 2, 255]),
        Value::Bytes(vec![]),
        Value::List(vec![Value::Int(1), Value::Text("two".into())]),
        Value::List(vec![]),
    ]
    .into_iter()
    .chain(float_values())
    .collect()
}

fn float_values() -> impl Iterator<Item = Value> {
    [
        std::f64::consts::PI,
        0.0,
        -0.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ]
    .into_iter()
    .map(|value| Value::Float(Float::new(value)))
}
#[test]
fn encode_decode_all_types() -> TestResult {
    scalar_values()
        .into_iter()
        .try_for_each(|value| -> TestResult {
            assert_eq!(decoded(&value)?, value);
            Ok(())
        })
}
#[test]
fn nan_roundtrips_to_canonical() -> TestResult {
    let value = Value::Float(Float::new(f64::NAN));
    match decoded(&value)? {
        Value::Float(f) => assert_eq!(f.bits(), 0x7FF8_0000_0000_0000),
        other => return Err(format!("expected float, got {other:?}").into()),
    }
    Ok(())
}
#[test]
fn time_zero_nanos_is_valid() -> TestResult {
    let value = Value::Time(Time::new(1_700_000_000, 0)?);
    assert_eq!(decoded(&value)?, value);
    Ok(())
}
#[test]
fn field_id_zero_rejected() {
    assert_eq!(FieldId::new(0), Err(RecordError::FieldIdZero { offset: 0 }));
}
#[test]
fn nanos_out_of_range_rejected() {
    assert_eq!(
        Time::new(0, 1_000_000_000),
        Err(RecordError::NanosOutOfRange {
            nanos: 1_000_000_000,
            offset: 0
        })
    );
}
#[test]
fn bytes_preserved_exactly() -> TestResult {
    let value = Value::Bytes((0..=255).collect());
    assert_eq!(decoded(&value)?, value);
    Ok(())
}
#[test]
fn to_value_from_borrowed() -> TestResult {
    let value = Value::Text("hello".into());
    assert_eq!(decoded(&value)?, value);
    Ok(())
}
#[test]
fn float_bits_accessible() {
    assert_eq!(
        Float::new(std::f64::consts::PI).bits(),
        std::f64::consts::PI.to_bits()
    );
}
#[test]
fn time_fields_accessible() -> TestResult {
    let time = Time::new(1_700_000_000, 123_456_789)?;
    assert_eq!(time.seconds(), 1_700_000_000);
    assert_eq!(time.nanoseconds(), 123_456_789);
    Ok(())
}
#[test]
fn generic_indefinite_root_is_accepted_but_rejected_when_read() -> TestResult {
    let record = Record::from_bytes(vec![0xBF, 1, 0, 0xFF])?;
    assert_eq!(record.as_bytes(), [0xBF, 1, 0, 0xFF]);
    assert_eq!(
        ValueRef::Map(record.root()).to_value(),
        Err(RecordError::IndefiniteLength { offset: 0 })
    );
    Ok(())
}
#[test]
fn iteration_terminates_after_reporting_an_unsupported_item() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 1, 0x83, 0xF6, 0xFA, 0, 0, 0, 0, 1])?;
    let list = list_field(&record)?;

    let mut items = list.iter()?;
    assert_eq!(
        items
            .next()
            .transpose()?
            .map(|v| v.to_value())
            .transpose()?,
        Some(Value::Null)
    );
    assert!(matches!(
        items.next(),
        Some(Err(RecordError::FloatType {
            type_name: "f32",
            ..
        }))
    ));
    assert!(items.next().is_none());
    assert!(items.next().is_none());
    Ok(())
}
fn list_field(record: &Record) -> TestResult<ListRef<'_>> {
    match record.root().get_field(FieldId::new(1)?)? {
        Some(ValueRef::List(list)) => Ok(list),
        other => Err(format!("expected list, got {other:?}").into()),
    }
}
#[test]
fn map_iteration_terminates_after_reporting_an_unsupported_value() -> TestResult {
    let record = Record::from_bytes(vec![0xA2, 1, 0xFA, 0, 0, 0, 0, 2, 0])?;

    let mut entries = record.root().iter()?;
    assert!(matches!(
        entries.next(),
        Some(Err(RecordError::FloatType {
            type_name: "f32",
            offset: 2
        }))
    ));
    assert!(entries.next().is_none());
    Ok(())
}
fn all_types() -> TestResult<Value> {
    let values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(i64::MIN),
        Value::Int(i64::MAX),
        Value::Float(Float::new(-0.0)),
        Value::Float(Float::new(f64::NAN)),
        Value::Float(Float::new(f64::INFINITY)),
        Value::Float(Float::new(f64::NEG_INFINITY)),
        Value::Text("café 🚀".into()),
        Value::Bytes((0..=255).collect()),
        Value::Time(Time::new(-1, 999_999_999)?),
        Value::List(vec![]),
        Value::Map(BTreeMap::new()),
        wrap_in_map(Value::Int(42)),
    ];
    Ok(Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(u32::MAX)?),
        Value::List(values),
    )])))
}
#[test]
fn all_value_types_roundtrip() -> TestResult {
    let value = all_types()?;
    let record = Record::encode(&value)?;
    assert_eq!(owned(&Record::from_bytes(record.into_bytes())?)?, value);
    Ok(())
}
#[test]
fn nan_canonicalization() -> TestResult {
    let left = wrap_in_map(Value::Float(Float::new(f64::NAN)));
    let right = wrap_in_map(Value::Float(Float::new(f64::from_bits(
        0x7FF0_0001_0000_0000,
    ))));
    assert_eq!(
        Record::encode(&left)?.as_bytes(),
        Record::encode(&right)?.as_bytes()
    );
    Ok(())
}
#[test]
fn signed_zero_distinct() -> TestResult {
    let left = wrap_in_map(Value::Float(Float::new(0.0)));
    let right = wrap_in_map(Value::Float(Float::new(-0.0)));
    assert_ne!(
        Record::encode(&left)?.as_bytes(),
        Record::encode(&right)?.as_bytes()
    );
    Ok(())
}
