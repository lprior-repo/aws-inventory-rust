use super::*;
use crate::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn part(fields: impl IntoIterator<Item = (u32, Value)>) -> Result<Record, crate::record::RecordError> {
    let fields = fields.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
    Record::encode(&Value::Map(fields))
}

#[test]
fn given_context_parts_when_decoded_then_only_known_context_tags_are_accepted() -> TestResult {
    [(2, IdentityPart::Partition), (3, IdentityPart::Account), (4, IdentityPart::Region)]
        .into_iter().try_for_each(|(kind, expected)| -> TestResult {
            let record = part([(1, Value::Int(kind))])?;
            assert_eq!(identity_part(ValueRef::Map(record.root()))?, expected);
            let extra = part([(1, Value::Int(kind)), (2, Value::Text("record-derived".to_owned()))])?;
            assert!(identity_part(ValueRef::Map(extra.root())).is_err());
            Ok(())
        })?;
    let unknown = part([(1, Value::Int(5))])?;
    assert!(identity_part(ValueRef::Map(unknown.root())).is_err());
    Ok(())
}

#[test]
fn given_selected_field_part_when_decoded_then_scalar_self_is_distinct_from_context() -> TestResult {
    let record = part([(1, Value::Int(1)), (2, Value::List(Vec::new()))])?;
    assert_eq!(identity_part(ValueRef::Map(record.root()))?, IdentityPart::Field(Box::new([])));
    let invalid = part([(1, Value::Int(1)), (2, Value::List(vec![Value::Int(0)]))])?;
    assert!(identity_part(ValueRef::Map(invalid.root())).is_err());
    Ok(())
}
