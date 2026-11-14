use inventory_core::record::*;
use std::collections::BTreeMap;

#[path = "content_hash/typed_reads.rs"]
mod typed_reads;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Entries = Vec<(Key, Value)>;

const OLDER: [u8; 15] = [
    0xA3, 1, 0x18, 42, 2, 0x65, b'h', b'e', b'l', b'l', b'o', 3, 0xA1, 7, 9,
];

fn old_record() -> TestResult<Record> {
    let record = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(42)),
        (Key::Field(FieldId::new(2)?), Value::Text("hello".into())),
        (Key::Field(FieldId::new(3)?), nested_map()?),
    ])))?;
    assert_eq!(record.as_bytes(), OLDER);
    Ok(record)
}

fn nested_map() -> TestResult<Value> {
    Ok(Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(7)?),
        Value::Int(9),
    )])))
}

fn old_entries() -> TestResult<Entries> {
    Ok(vec![
        (Key::Field(FieldId::new(1)?), Value::Int(42)),
        (Key::Field(FieldId::new(2)?), Value::Text("hello".into())),
        (Key::Field(FieldId::new(3)?), nested_map()?),
    ])
}

fn entries(root: MapRef<'_>) -> Result<Entries, RecordError> {
    root.iter()?
        .map(|entry| {
            let (key, value) = entry?;
            Ok((key.to_key(), value.to_value()?))
        })
        .collect()
}

fn borrowed_text<'a>(root: MapRef<'a>) -> TestResult<&'a str> {
    let Some(ValueRef::Text(text)) = root.get_typed(FieldId::new(2)?, ValueKind::Text)? else {
        return Err("missing borrowed text".into());
    };
    Ok(text)
}

fn typed(root: MapRef<'_>, id: u32, kind: ValueKind) -> Result<Option<Value>, RecordError> {
    root.get_typed(FieldId::new(id)?, kind)?
        .map(|value| value.to_value())
        .transpose()
}

fn assert_later_reads(root: MapRef<'_>) -> TestResult {
    assert_eq!(typed(root, 1, ValueKind::Int)?, Some(Value::Int(42)));
    assert_eq!(
        typed(root, 2, ValueKind::Text)?,
        Some(Value::Text("hello".into()))
    );
    assert_eq!(typed(root, 3, ValueKind::Map)?, Some(nested_map()?));
    assert_eq!(typed(root, 4, ValueKind::Int)?, None);
    assert_eq!(
        typed(root, 2, ValueKind::Int).err(),
        Some(RecordError::KindMismatch {
            field: FieldId::new(2)?,
            expected: ValueKind::Int,
            found: ValueKind::Text,
        })
    );
    Ok(())
}

fn same_validation_error(bytes: &[u8], error: RecordError) -> TestResult {
    assert_eq!(Record::from_bytes(bytes.to_vec()), Err(error));
    assert_eq!(MapRef::from_bytes(bytes).err(), Some(error));
    Ok(())
}

#[test]
fn added_registry_field_is_absent_in_older_record() -> TestResult {
    let record = old_record()?;

    assert!(matches!(
        record.root().get_typed(FieldId::new(4)?, ValueKind::Int),
        Ok(None)
    ));
    Ok(())
}

#[test]
fn removed_field_remains_iterable_without_blocking_other_lookups() -> TestResult {
    let record = old_record()?;

    assert_eq!(entries(record.root())?, old_entries()?);

    assert_eq!(
        typed(record.root(), 1, ValueKind::Int)?,
        Some(Value::Int(42))
    );
    assert_eq!(
        typed(record.root(), 3, ValueKind::Map)?,
        Some(nested_map()?)
    );
    Ok(())
}

#[test]
fn changed_registry_type_reports_both_kinds_and_field_id() -> TestResult {
    let record = old_record()?;

    assert_eq!(
        typed(record.root(), 2, ValueKind::Int),
        Err(RecordError::KindMismatch {
            field: FieldId::new(2)?,
            expected: ValueKind::Int,
            found: ValueKind::Text,
        })
    );
    Ok(())
}

#[test]
fn null_is_not_a_missing_or_text_value() -> TestResult {
    let record = Record::from_bytes(vec![0xA1, 2, 0xF6])?;

    assert_eq!(
        typed(record.root(), 2, ValueKind::Text),
        Err(RecordError::KindMismatch {
            field: FieldId::new(2)?,
            expected: ValueKind::Text,
            found: ValueKind::Null,
        })
    );
    assert_eq!(typed(record.root(), 2, ValueKind::Null)?, Some(Value::Null));
    Ok(())
}

#[test]
fn owned_offset_record_and_direct_root_have_identical_typed_reads() -> TestResult {
    let original = old_record()?;
    let mut buffer = vec![0xFF];
    buffer.extend_from_slice(original.as_bytes());

    let slice = buffer.get(1..).ok_or("missing offset slice")?;
    let copied = Record::from_bytes(slice.to_vec())?;
    assert_later_reads(copied.root())?;
    assert_later_reads(original.root())?;
    Ok(())
}

#[test]
fn borrowed_offset_view_preserves_values_and_borrows_original_text() -> TestResult {
    let original = old_record()?;
    let length = original.as_bytes().len();
    let end = length.checked_add(1).ok_or("offset overflow")?;
    let mut buffer = vec![0xFF];
    buffer.extend_from_slice(original.as_bytes());
    buffer.push(0xFF);

    let slice = buffer.get(1..end).ok_or("missing borrowed slice")?;
    let borrowed = MapRef::from_bytes(slice)?;
    assert_later_reads(borrowed)?;
    let text = borrowed_text(borrowed)?;

    assert_eq!(text, "hello");
    let stored_text = slice.get(6..11).ok_or("text slice")?;
    assert_eq!(text.as_ptr(), stored_text.as_ptr());
    Ok(())
}

#[test]
fn borrowed_trailing_bytes_error_matches_owned_validation() -> TestResult {
    same_validation_error(&[0xA0, 0xF6], RecordError::NotOneItem { offset: 1 })
}

#[test]
fn borrowed_non_map_error_matches_owned_validation() -> TestResult {
    same_validation_error(&[0x01], RecordError::NotMap)
}

#[test]
fn borrowed_truncated_error_matches_owned_validation() -> TestResult {
    let truncated = [0xA1, 1];
    let owned = Record::from_bytes(truncated.to_vec()).err();
    assert!(matches!(owned, Some(RecordError::Missing { .. })));
    assert_eq!(MapRef::from_bytes(&truncated).err(), owned);
    Ok(())
}

#[test]
fn borrowed_oversize_error_matches_owned_validation() -> TestResult {
    let length = MAX_RECORD_BYTES.checked_add(1).ok_or("size overflow")?;
    let bytes = vec![0; length];

    same_validation_error(&bytes, RecordError::SizeLimit)
}
