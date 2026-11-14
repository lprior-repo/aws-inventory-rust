//! Original record scenarios, migrated to enum constructors and strict boundaries.
use inventory_core::record::*;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const WORKED: [u8; 14] = [
    0xA4, 1, 0x20, 2, 0x62, b'h', b'i', 3, 0x82, 0xF5, 0xF6, 0x61, b'k', 0,
];
fn wrap_in_map(value: Value) -> Value {
    Value::Map(BTreeMap::from([(Key::Name("value".into()), value)]))
}
fn owned(record: &Record) -> Result<Value, RecordError> {
    ValueRef::Map(record.root()).to_value()
}
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
fn worked_example_exact_bytes() -> TestResult {
    let value = worked_value()?;
    let record = Record::encode(&value)?;
    assert_eq!(record.as_bytes(), WORKED);
    assert_eq!(owned(&Record::from_bytes(WORKED.to_vec())?)?, value);
    Ok(())
}
#[test]
fn get_field_lookup_positions() -> TestResult {
    let entries = [10, 50, 100]
        .into_iter()
        .map(|n| Ok((Key::Field(FieldId::new(n)?), Value::Int(i64::from(n)))))
        .collect::<Result<BTreeMap<_, _>, RecordError>>()?;
    let record = Record::encode(&Value::Map(entries.clone()))?;
    [1, 10, 30, 50, 100, 200]
        .into_iter()
        .try_for_each(|n| -> TestResult {
            let id = FieldId::new(n)?;
            let found = record
                .root()
                .get_field(id)?
                .map(|v| v.to_value())
                .transpose()?;
            assert_eq!(found.as_ref(), entries.get(&Key::Field(id)));
            Ok(())
        })
}
#[test]
fn get_name_lookup_positions() -> TestResult {
    let entries = BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(1)),
        (Key::Name("a".into()), Value::Int(2)),
        (Key::Name("m".into()), Value::Int(3)),
        (Key::Name("z".into()), Value::Int(4)),
    ]);
    let record = Record::encode(&Value::Map(entries.clone()))?;
    ["", "a", "b", "m", "q", "z", "zz"]
        .into_iter()
        .try_for_each(|name| -> TestResult {
            let found = record
                .root()
                .get_name(name)?
                .map(|v| v.to_value())
                .transpose()?;
            assert_eq!(found.as_ref(), entries.get(&Key::Name(name.into())));
            Ok(())
        })
}
#[test]
fn lookup_skips_nested_containers() -> TestResult {
    let value = Value::Map(BTreeMap::from([
        (
            Key::Field(FieldId::new(1)?),
            wrap_in_map(Value::List(vec![Value::Null])),
        ),
        (Key::Field(FieldId::new(2)?), Value::Text("target".into())),
    ]));
    let record = Record::encode(&value)?;
    let found = record
        .root()
        .get_field(FieldId::new(2)?)?
        .ok_or("missing target")?;
    assert_eq!(found.to_value()?, Value::Text("target".into()));
    Ok(())
}
fn nested_lists(containers: usize) -> Value {
    wrap_in_map((1..containers).fold(Value::Null, |v, _| Value::List(vec![v])))
}
fn nested_maps(containers: usize) -> Value {
    (0..containers).fold(Value::Null, |v, _| wrap_in_map(v))
}
#[test]
fn depth_32_lists_encode() -> TestResult {
    let value = nested_lists(32);
    assert_eq!(
        owned(&Record::from_bytes(Record::encode(&value)?.into_bytes())?)?,
        value
    );
    Ok(())
}
#[test]
fn depth_33_lists_refused_encode() {
    assert_eq!(
        Record::encode(&nested_lists(33)),
        Err(RecordError::DepthLimit)
    );
}
#[test]
fn depth_32_maps_encode() -> TestResult {
    let value = nested_maps(32);
    assert_eq!(
        owned(&Record::from_bytes(Record::encode(&value)?.into_bytes())?)?,
        value
    );
    Ok(())
}
#[test]
fn depth_33_maps_refused_encode() {
    assert_eq!(
        Record::encode(&nested_maps(33)),
        Err(RecordError::DepthLimit)
    );
}
fn sized_value(extra: usize) -> TestResult<Value> {
    let probe_size = 65_536;
    let probe = Record::encode(&wrap_in_map(Value::Bytes(vec![0; probe_size])))?;
    let overhead = probe
        .as_bytes()
        .len()
        .checked_sub(probe_size)
        .ok_or("overhead")?;
    let payload = MAX_RECORD_BYTES
        .checked_sub(overhead)
        .and_then(|n| n.checked_add(extra))
        .ok_or("size")?;
    Ok(wrap_in_map(Value::Bytes(vec![0; payload])))
}
#[test]
fn size_16mib_accepted() -> TestResult {
    let record = Record::encode(&sized_value(0)?)?;
    assert_eq!(record.as_bytes().len(), MAX_RECORD_BYTES);
    assert_eq!(Record::from_bytes(record.as_bytes().to_vec())?, record);
    Ok(())
}
#[test]
fn size_16mib_plus_1_refused() -> TestResult {
    assert_eq!(
        Record::encode(&sized_value(1)?),
        Err(RecordError::SizeLimit)
    );
    Ok(())
}
#[test]
fn from_bytes_empty_refused() {
    assert!(matches!(
        Record::from_bytes(vec![]),
        Err(RecordError::Missing { .. })
    ));
}
#[test]
fn from_bytes_prefixes_refused() -> TestResult {
    (0..WORKED.len()).try_for_each(|length| -> TestResult {
        let prefix = WORKED.get(..length).ok_or("prefix")?;
        assert!(matches!(
            Record::from_bytes(prefix.to_vec()),
            Err(RecordError::Missing { .. })
        ));
        Ok(())
    })
}
#[test]
fn from_bytes_trailing_refused() {
    let bytes = WORKED.into_iter().chain([0xF6]).collect();
    assert_eq!(
        Record::from_bytes(bytes),
        Err(RecordError::NotOneItem { offset: 14 })
    );
}
#[test]
fn from_bytes_top_array_refused() {
    assert_eq!(Record::from_bytes(vec![0x81, 0]), Err(RecordError::NotMap));
}
fn read_error(bytes: Vec<u8>) -> TestResult<RecordError> {
    let record = Record::from_bytes(bytes)?;
    owned(&record)
        .err()
        .ok_or_else(|| "expected a read error".into())
}
#[test]
fn read_refuses_f32() -> TestResult {
    assert!(matches!(
        read_error(vec![0xA1, 1, 0xFA, 0, 0, 0, 0])?,
        RecordError::FloatType {
            type_name: "f32",
            offset: 2
        }
    ));
    Ok(())
}
#[test]
fn read_refuses_indefinite_array() -> TestResult {
    assert_eq!(
        read_error(vec![0xA1, 1, 0x9F, 0xFF])?,
        RecordError::IndefiniteLength { offset: 2 }
    );
    Ok(())
}
#[test]
fn read_refuses_tag_2() -> TestResult {
    assert_eq!(
        read_error(vec![0xA1, 1, 0xC2, 0x60])?,
        RecordError::Tag { tag: 2, offset: 2 }
    );
    Ok(())
}
#[test]
fn read_refuses_bool_key() -> TestResult {
    assert_eq!(
        read_error(vec![0xA1, 0xF5, 0])?,
        RecordError::KeyType {
            found: minicbor::data::Type::Bool,
            offset: 1
        }
    );
    Ok(())
}
#[test]
fn read_refuses_field_id_zero() -> TestResult {
    assert_eq!(
        read_error(vec![0xA1, 0, 0])?,
        RecordError::FieldIdZero { offset: 1 }
    );
    Ok(())
}
#[test]
fn read_refuses_large_nanos() -> TestResult {
    assert_eq!(
        read_error(vec![0xA1, 1, 0xC1, 0x82, 0, 0x1A, 0x3B, 0x9A, 0xCA, 0])?,
        RecordError::NanosOutOfRange {
            nanos: 1_000_000_000,
            offset: 5
        }
    );
    Ok(())
}
#[test]
fn read_refuses_invalid_utf8() -> TestResult {
    // Given invalid text, when structural validation skips it, then retain the UTF-8 cause.
    let error = Record::from_bytes(vec![0xA1, 1, 0x61, 0x80])
        .err()
        .ok_or("accepted invalid text")?;
    match error {
        RecordError::NotUtf8 { offset, source } => {
            assert_eq!(offset, 2);
            assert_eq!(source.valid_up_to(), 0);
            assert_eq!(source.error_len(), Some(1));
        }
        other => return Err(format!("wrong error: {other:?}").into()),
    }
    Ok(())
}
#[test]
fn lookup_stops_before_invalid_keys_after_the_requested_sorted_position() -> TestResult {
    // Given a larger valid key followed by an unsupported key.
    let record = Record::from_bytes(vec![0xA2, 2, 0, 0xF5, 0])?;
    // When seeking a smaller field, then no later key or value is interpreted.
    assert!(record.root().get_field(FieldId::new(1)?)?.is_none());
    let record = Record::from_bytes(vec![0xA2, 0x61, b'z', 0, 0xF5, 0])?;
    assert!(record.root().get_name("a")?.is_none());
    Ok(())
}
#[test]
fn null_advances_before_following_list_items_and_map_keys() -> TestResult {
    // Given adjacent nulls and following scalars in both containers.
    let value = Value::Map(BTreeMap::from([
        (Key::Name("a".into()), Value::Null),
        (
            Key::Name("b".into()),
            Value::List(vec![Value::Null, Value::Int(7), Value::Null]),
        ),
        (Key::Name("c".into()), Value::Int(8)),
    ]));
    // When iterators rebuild the tree, then no null repeats or hides the next item.
    assert_eq!(owned(&Record::encode(&value)?)?, value);
    Ok(())
}
