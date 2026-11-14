mod cbor_support;

use cbor_support::{TestResult, envelope, owned};
use inventory_core::record::{FieldId, Record, RecordError, Value};

const WORKED: [u8; 14] = [
    0xA4, 0x01, 0x20, 0x02, 0x62, 0x68, 0x69, 0x03, 0x82, 0xF5, 0xF6, 0x61, 0x6B, 0x00,
];

fn read_field_one(record: &Record) -> Result<Option<Value>, RecordError> {
    record
        .root()
        .get_field(FieldId::new(1)?)?
        .map(|value| value.to_value())
        .transpose()
}

#[test]
fn empty_input_is_rejected_as_missing_cbor() {
    assert!(matches!(
        Record::from_bytes(Vec::new()),
        Err(RecordError::Missing { .. })
    ));
}

#[test]
fn every_strict_prefix_of_the_worked_example_is_rejected() -> TestResult {
    (0..WORKED.len()).try_for_each(|length| -> TestResult {
        let prefix = WORKED.get(..length).ok_or("prefix bounds")?;

        assert!(
            matches!(
                Record::from_bytes(prefix.to_vec()),
                Err(RecordError::Missing { .. })
            ),
            "prefix length {length}"
        );
        Ok(())
    })
}

#[test]
fn trailing_item_is_rejected_at_the_end_of_the_first_record() {
    let bytes = WORKED.into_iter().chain([0xF6]).collect();

    assert_eq!(
        Record::from_bytes(bytes),
        Err(RecordError::NotOneItem { offset: 14 })
    );
}

#[test]
fn top_level_array_is_not_a_record_even_when_well_formed() {
    assert_eq!(Record::from_bytes(vec![0x80]), Err(RecordError::NotMap));
}

#[test]
fn encoding_a_scalar_or_list_is_rejected_as_a_nonmap_record() {
    [Value::Null, Value::Int(1), Value::List(vec![])]
        .iter()
        .for_each(|value| {
            assert_eq!(Record::encode(value), Err(RecordError::NotMap));
        });
}

#[test]
fn f32_is_well_formed_but_reading_names_the_unsupported_float_width() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 1, 0xFA, 0, 0, 0, 0])?;

    assert!(matches!(
        read_field_one(&record),
        Err(RecordError::FloatType {
            type_name: "f32",
            ..
        })
    ));
    assert!(matches!(
        owned(&record),
        Err(RecordError::FloatType {
            type_name: "f32",
            ..
        })
    ));
    Ok(())
}

#[test]
fn indefinite_array_is_well_formed_but_reading_names_the_length_form() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 1, 0x9F, 0xFF])?;

    assert!(matches!(
        read_field_one(&record),
        Err(RecordError::IndefiniteLength { .. })
    ));
    assert!(matches!(
        owned(&record),
        Err(RecordError::IndefiniteLength { .. })
    ));
    Ok(())
}

#[test]
fn tag_two_is_well_formed_but_reading_names_the_unsupported_tag() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 1, 0xC2, 0x40])?;

    assert!(matches!(
        read_field_one(&record),
        Err(RecordError::Tag { tag: 2, .. })
    ));
    assert!(matches!(
        owned(&record),
        Err(RecordError::Tag { tag: 2, .. })
    ));
    Ok(())
}

#[test]
fn boolean_map_key_is_rejected_during_validation() {
    assert_eq!(
        Record::from_bytes(vec![0xA1, 0xF5, 0]),
        Err(RecordError::KeyType {
            found: minicbor::data::Type::Bool,
            offset: 1
        })
    );
}

#[test]
fn zero_field_id_is_rejected_during_validation() {
    assert_eq!(
        Record::from_bytes(vec![0xA1, 0, 0]),
        Err(RecordError::FieldIdZero { offset: 1 })
    );
}

#[test]
fn one_billion_nanoseconds_is_well_formed_but_reading_names_the_range_error() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 1, 0xC1, 0x82, 0, 0x1A, 0x3B, 0x9A, 0xCA, 0])?;

    assert!(matches!(
        read_field_one(&record),
        Err(RecordError::NanosOutOfRange {
            nanos: 1_000_000_000,
            ..
        })
    ));
    assert!(matches!(
        owned(&record),
        Err(RecordError::NanosOutOfRange {
            nanos: 1_000_000_000,
            ..
        })
    ));
    Ok(())
}

#[test]
fn invalid_utf8_is_rejected_during_minicbor_structural_validation() {
    [vec![0xA1, 1, 0x61, 0x80], vec![0xA1, 0x61, 0x80, 0]]
        .into_iter()
        .for_each(|bytes| {
            assert!(matches!(
                Record::from_bytes(bytes),
                Err(RecordError::NotUtf8 { .. })
            ));
        });
}

#[test]
fn valid_multibyte_utf8_survives_validation_and_reading() -> TestResult {
    let value = envelope(Value::Text("é🦀".into()));

    let record = Record::from_bytes(Record::encode(&value)?.into_bytes())?;

    assert_eq!(owned(&record)?, value);
    Ok(())
}
