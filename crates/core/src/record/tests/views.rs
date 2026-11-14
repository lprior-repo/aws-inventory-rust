use super::{Record, RecordError, TestResult};
use crate::record::{FieldId, ValueRef};
use minicbor::data::Type;

fn target_after(payload: impl IntoIterator<Item = u8>) -> TestResult<Record> {
    let bytes = [0xA2, 1].into_iter().chain(payload).chain([2, 3]).collect();
    Ok(Record::from_bytes(bytes)?)
}

fn assert_target(record: &Record) -> TestResult {
    match record.root().get_field(FieldId::new(2)?)? {
        Some(ValueRef::Int(value)) => assert_eq!(value, 3),
        other => return Err(format!("wrong sibling: {other:?}").into()),
    }
    Ok(())
}

#[test]
fn lookup_skips_nested_indefinite_forms_without_allocating_a_skip_stack() -> TestResult {
    // Given the mixed sequence that would make a generic library container skip allocate.
    let record = Record::from_bytes(vec![0xA2, 1, 0x82, 0x9F, 0xFF, 0, 2, 3])?;
    // When looking up field two, then unselected unsupported forms do not interfere.
    assert_target(&record)
}

#[test]
fn list_access_skips_unselected_indefinite_children_but_rejects_selected_ones() -> TestResult {
    // Given a definite list containing an unsupported indefinite child and a scalar.
    let record = Record::from_bytes(vec![0xA1, 1, 0x82, 0x9F, 0xFF, 0])?;
    let Some(ValueRef::List(list)) = record.root().get_field(FieldId::new(1)?)? else {
        return Err("expected list".into());
    };
    // When selecting the sibling, then it is readable; selecting the child still fails.
    assert!(matches!(list.get(1)?, Some(ValueRef::Int(0))));
    assert!(matches!(
        list.get(0),
        Err(RecordError::IndefiniteLength { offset: 1 })
    ));
    Ok(())
}

fn mixed_groups() -> [Vec<u8>; 4] {
    [
        vec![0x82, 0x9F, 0xFF, 0],
        vec![
            0x82, 0xBF, 0x81, 0, 0x9F, 0xA1, 0xF5, 0xFA, 0, 0, 0, 0, 0xFF, 0xFF, 0,
        ],
        vec![
            0x9F, 0x82, 0x9F, 0xFF, 0, 0xBF, 0x61, b'k', 0xD8, 99, 0x9F, 0xFF, 0xFF, 0xFF,
        ],
        vec![0x9F, 0x7F, 0x61, b'x', 0xFF, 0x5F, 0x41, 0xFF, 0xFF, 0xFF],
    ]
}

#[test]
fn generic_skip_preserves_boundaries_in_mixed_definite_and_indefinite_groups() -> TestResult {
    // Given nested groups, generic keys/tags/floats, and chunked strings/bytes.
    // When each entire unselected value is skipped, then field two retains its header.
    mixed_groups()
        .into_iter()
        .try_for_each(|payload| assert_target(&target_after(payload)?))
}

#[test]
fn unknown_tags_are_skipped_unselected_but_rejected_selected() -> TestResult {
    let record = target_after([0xD8, 99, 0])?;
    // Given an unknown tag, when skipped then the sibling is readable.
    assert_target(&record)?;
    // When selected, then the tag policy still applies.
    assert!(matches!(
        record.root().get_field(FieldId::new(1)?),
        Err(RecordError::Tag { .. })
    ));
    Ok(())
}

#[test]
fn narrow_floats_are_skipped_unselected_but_rejected_selected() -> TestResult {
    let record = target_after([0xFA, 0, 0, 0, 0])?;
    // Given f32, when skipped then the sibling is readable; selected it remains unsupported.
    assert_target(&record)?;
    assert!(matches!(
        record.root().get_field(FieldId::new(1)?),
        Err(RecordError::FloatType {
            type_name: "f32",
            ..
        })
    ));
    Ok(())
}

#[test]
fn generic_nested_keys_are_rejected_before_a_view_can_be_created() {
    // Given a reachable boolean-keyed map, when validating, then reject before any lookup.
    assert_eq!(
        Record::from_bytes(vec![0xA2, 1, 0xA1, 0xF5, 0, 2, 3]),
        Err(RecordError::KeyType {
            found: Type::Bool,
            offset: 3
        })
    );
}

#[test]
fn skipped_depth_thirty_three_is_not_subject_to_conversion_depth_policy() -> TestResult {
    // Given a root map plus 32 nested arrays, all structurally well formed.
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encoder.map(2)?.u8(1)?;
    (0..32).try_for_each(|_| encoder.array(1).map(|_| ()))?;
    encoder.null()?.u8(2)?.u8(3)?;
    let record = Record::from_bytes(encoder.into_writer())?;
    // When skipping then lookup succeeds; converting the whole record remains bounded.
    assert_target(&record)?;
    assert_eq!(
        ValueRef::Map(record.root()).to_value(),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn allocation_free_skip_keeps_supported_container_headers_and_sibling_positions() -> TestResult {
    // Given a nested list, nested map, timestamp, and a following scalar.
    let bytes = vec![
        0xA2, 1, 0x83, 0x81, 0xF6, 0xA1, 1, 0, 0xC1, 0x82, 0, 0, 2, 3,
    ];
    let record = Record::from_bytes(bytes)?;
    // When looking beyond the containers, then every header is skipped exactly once.
    assert_target(&record)
}
