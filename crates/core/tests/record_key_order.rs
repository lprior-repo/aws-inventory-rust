use inventory_core::record::{FieldId, KeyRef, MapRef, Record, RecordError, Value};
use minicbor::data::Type;

#[path = "record_key_order/properties.rs"]
mod properties;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn rejected(bytes: &[u8], error: RecordError) {
    assert_eq!(Record::from_bytes(bytes.to_vec()), Err(error));
    assert_eq!(MapRef::from_bytes(bytes).err(), Some(error));
}

#[test]
fn swapped_root_fields_are_rejected_at_the_second_key() {
    rejected(
        &[0xA2, 2, 0, 1, 0x61, b'x'],
        RecordError::KeyOutOfOrder { offset: 3 },
    );
}

#[test]
fn duplicate_field_keys_are_rejected_at_the_repeated_key() {
    rejected(&[0xA2, 1, 0, 1, 1], RecordError::DuplicateKey { offset: 3 });
}

#[test]
fn duplicate_name_keys_are_rejected_at_the_repeated_key() {
    rejected(
        &[0xA2, 0x61, b'a', 0, 0x61, b'a', 1],
        RecordError::DuplicateKey { offset: 4 },
    );
}

#[test]
fn name_before_field_is_rejected_at_the_field_key() {
    rejected(
        &[0xA2, 0x60, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 3 },
    );
}

#[test]
fn names_are_ordered_by_bytes_not_encoded_length() {
    rejected(
        &[0xA2, 0x61, b'z', 0, 0x62, b'a', b'a', 0],
        RecordError::KeyOutOfOrder { offset: 4 },
    );
}

#[test]
fn swapped_fields_in_map_values_are_rejected_at_the_nested_key() {
    rejected(
        &[0xA1, 1, 0xA2, 2, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 5 },
    );
}

#[test]
fn swapped_fields_in_list_items_are_rejected_at_the_nested_key() {
    rejected(
        &[0xA1, 1, 0x81, 0x81, 0xA2, 2, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 7 },
    );
}

#[test]
fn unsupported_reachable_keys_are_rejected_during_validation() {
    [
        (vec![0xA1, 0xF5, 0], 1),
        (vec![0xA1, 1, 0xA1, 0xF5, 0], 3),
        (vec![0xA1, 1, 0x81, 0xA1, 0xF5, 0], 4),
    ]
    .into_iter()
    .for_each(|(bytes, offset)| {
        rejected(
            &bytes,
            RecordError::KeyType {
                found: Type::Bool,
                offset,
            },
        );
    });
}

#[test]
fn differently_encoded_equal_field_ids_are_still_duplicates() {
    rejected(
        &[0xA2, 1, 0, 0x18, 1, 1],
        RecordError::DuplicateKey { offset: 3 },
    );
}

fn sibling_after(payload: &[u8]) -> TestResult {
    let bytes: Vec<_> = [0xA2, 1]
        .into_iter()
        .chain(payload.iter().copied())
        .chain([2, 7])
        .collect();
    let record = Record::from_bytes(bytes.clone())?;
    assert_eq!(record.as_bytes(), bytes);
    let borrowed = MapRef::from_bytes(&bytes)?;
    [record.root(), borrowed]
        .into_iter()
        .try_for_each(assert_sibling)
}

fn assert_sibling(map: MapRef<'_>) -> TestResult {
    let found = map.get_field(FieldId::new(2)?)?;
    assert_eq!(
        found.map(|v| v.to_value()).transpose()?,
        Some(Value::Int(7))
    );
    Ok(())
}

#[test]
fn unsorted_maps_inside_tags_remain_skippable_before_a_sibling() -> TestResult {
    sibling_after(&[0xD8, 99, 0xA2, 2, 0, 1, 0])
}

#[test]
fn opaque_keys_duplicates_and_nested_maps_remain_generic_inside_tags() -> TestResult {
    sibling_after(&[0xC1, 0x81, 0xA2, 0xF5, 0, 0xF5, 0xA2, 2, 0, 1, 0])
}

#[test]
fn unsorted_maps_inside_indefinite_containers_remain_skippable() -> TestResult {
    sibling_after(&[0x9F, 0xA2, 2, 0, 1, 0, 0xFF])?;
    sibling_after(&[0xBF, 0xF5, 0xA2, 2, 0, 1, 0, 0xFF])
}

#[test]
fn opaque_child_does_not_disable_validation_of_a_following_map() {
    rejected(
        &[0xA2, 1, 0xC2, 0xA2, 2, 0, 1, 0, 2, 0xA2, 2, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 12 },
    );
}

fn linear(map: MapRef<'_>, target: KeyRef<'_>) -> TestResult<Option<Value>> {
    map.iter()?.try_fold(None, |found, entry| {
        let (key, value) = entry?;
        if key == target {
            Ok(Some(value.to_value()?))
        } else {
            Ok(found)
        }
    })
}

fn lookup(map: MapRef<'_>, key: KeyRef<'_>) -> TestResult<Option<Value>> {
    let found = match key {
        KeyRef::Field(id) => map.get_field(id)?,
        KeyRef::Name(name) => map.get_name(name)?,
    };
    Ok(found.map(|value| value.to_value()).transpose()?)
}

#[test]
fn validated_nonshortest_keys_have_lookups_equal_to_a_full_scan() -> TestResult {
    let bytes = [
        0xA5, 0x18, 1, 0, 0x18, 24, 1, 0x60, 2, 0x62, b'a', b'a', 3, 0x62, 0xC3, 0xA9, 4,
    ];
    let map = MapRef::from_bytes(&bytes)?;

    compare_fields(map)?;
    compare_names(map)
}

fn compare_fields(map: MapRef<'_>) -> TestResult {
    [1, 2, 24, 25, u32::MAX]
        .into_iter()
        .try_for_each(|id| -> TestResult {
            let key = KeyRef::Field(FieldId::new(id)?);
            assert_eq!(lookup(map, key)?, linear(map, key)?);
            Ok(())
        })
}

fn compare_names(map: MapRef<'_>) -> TestResult {
    ["", "a", "aa", "z", "é", "ê"]
        .into_iter()
        .try_for_each(|name| -> TestResult {
            let key = KeyRef::Name(name);
            assert_eq!(lookup(map, key)?, linear(map, key)?);
            Ok(())
        })
}

#[test]
fn overflowing_map_arity_retains_the_generic_framing_error() {
    rejected(
        &[0xBB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        RecordError::Malformed { offset: 9 },
    );
}

#[test]
fn duplicate_fields_in_map_values_are_rejected_at_the_repeated_key() {
    rejected(
        &[0xA1, 1, 0xA2, 1, 0, 1, 0],
        RecordError::DuplicateKey { offset: 5 },
    );
}

#[test]
fn duplicate_fields_in_list_items_are_rejected_at_the_repeated_key() {
    rejected(
        &[0xA1, 1, 0x81, 0xA2, 1, 0, 1, 0],
        RecordError::DuplicateKey { offset: 6 },
    );
}

#[test]
fn mixed_width_field_keys_are_ordered_by_decoded_identifier() -> TestResult {
    let bytes = [0xA2, 0x18, 1, 0, 2, 0];
    let owned = Record::from_bytes(bytes.to_vec())?;
    assert_eq!(owned.as_bytes(), bytes);
    let borrowed = MapRef::from_bytes(&bytes)?;
    [owned.root(), borrowed]
        .into_iter()
        .try_for_each(assert_zero_fields)?;

    rejected(
        &[0xA2, 2, 0, 0x18, 1, 0],
        RecordError::KeyOutOfOrder { offset: 3 },
    );
    Ok(())
}

fn assert_zero_fields(map: MapRef<'_>) -> TestResult {
    [1, 2].into_iter().try_for_each(|number| -> TestResult {
        let key = KeyRef::Field(FieldId::new(number)?);
        assert_eq!(lookup(map, key)?, Some(Value::Int(0)));
        Ok(())
    })
}

#[test]
fn strict_map_order_resumes_after_an_indefinite_array_child() {
    rejected(
        &[0xA2, 1, 0x9F, 0xFF, 2, 0xA2, 2, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 8 },
    );
}

#[test]
fn strict_map_order_resumes_after_an_indefinite_map_child() {
    rejected(
        &[0xA2, 1, 0xBF, 0xFF, 2, 0xA2, 2, 0, 1, 0],
        RecordError::KeyOutOfOrder { offset: 8 },
    );
}
