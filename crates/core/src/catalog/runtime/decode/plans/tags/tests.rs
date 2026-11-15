use super::*;
use crate::record::{Key, Record, Value};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn decoded(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Result<TagPlan, RuntimeCatalogError>> {
    let values = entries.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
    let bytes = Record::encode(&Value::Map(values))?.into_bytes();
    Ok(parse(MapRef::from_bytes_bounded(&bytes)?))
}

fn path(ids: impl IntoIterator<Item = i64>) -> Value {
    Value::List(ids.into_iter().map(Value::Int).collect())
}

#[test]
fn given_numeric_tags_when_decoded_then_exact_ids_and_root_are_retained() -> TestResult {
    assert_eq!(decoded([(1, Value::Int(0)), (2, path([7, 9])), (3, Value::Int(11)), (4, Value::Int(12))])??,
        TagPlan::List { path: vec![FieldId::new(7)?, FieldId::new(9)?].into_boxed_slice(), key: FieldId::new(11)?, value: FieldId::new(12)? });
    assert_eq!(decoded([(1, Value::Int(1)), (2, path([]))])??, TagPlan::Map { path: Box::new([]) });
    assert_eq!(decoded([(1, Value::Int(1)), (2, path(1..=8))])??,
        TagPlan::Map { path: (1..=8).map(FieldId::new).collect::<Result<Vec<_>, _>>()?.into_boxed_slice() });
    Ok(())
}

#[test]
fn given_malformed_tag_variants_when_decoded_then_no_plan_is_minted() -> TestResult {
    let invalid = [
        vec![(1, Value::Int(2)), (2, path([]))],
        vec![(1, Value::Int(0)), (2, path([])), (3, Value::Int(1))],
        vec![(1, Value::Int(0)), (2, path([])), (3, Value::Int(1)), (4, Value::Int(1))],
        vec![(1, Value::Int(1)), (2, path([])), (3, Value::Int(1))],
        vec![(1, Value::Int(1)), (2, Value::Null)],
        vec![(1, Value::Int(1)), (2, path([-1]))],
        vec![(1, Value::Int(1)), (2, path([i64::MAX]))],
        vec![(1, Value::Int(0)), (2, path([])), (3, Value::Int(1)), (4, Value::Text("Value".to_owned()))],
    ];
    invalid.into_iter().try_for_each(|fields| {
        assert!(matches!(decoded(fields)?, Err(RuntimeCatalogError::Capability)));
        Ok(())
    })
}

#[test]
fn given_tag_path_or_member_numeric_bounds_when_decoded_then_exact_failure_is_retained() -> TestResult {
    assert!(matches!(decoded([(1, Value::Int(1)), (2, path(1..=9))])?,
        Err(RuntimeCatalogError::Bounds)));
    assert!(matches!(decoded([(1, Value::Int(1)), (2, path([0]))])?,
        Err(RuntimeCatalogError::Record(crate::record::RecordError::FieldIdZero { .. }))));
    assert!(matches!(decoded([(1, Value::Int(0)), (2, path([])), (3, Value::Int(0)), (4, Value::Int(2))])?,
        Err(RuntimeCatalogError::Record(crate::record::RecordError::FieldIdZero { .. }))));
    Ok(())
}
