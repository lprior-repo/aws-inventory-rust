use super::*;
use crate::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> Result<Value, crate::record::RecordError> {
    entries.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, _>>().map(Value::Map)
}

fn token_value() -> Result<Value, crate::record::RecordError> {
    let path = Value::List(vec![Value::Int(1)]);
    let placement = map([(1, Value::Int(3)), (2, path.clone())])?;
    map([(1, path.clone()), (2, path), (3, Value::Int(0)),
        (4, placement.clone()), (5, placement)])
}

fn with_optional(value: Value) -> Result<Record, crate::record::RecordError> {
    let Value::Map(mut fields) = token_value()? else {
        return Err(crate::record::RecordError::NotMap);
    };
    fields.insert(Key::Field(FieldId::new(6)?), value);
    Record::encode(&Value::Map(fields))
}

#[test]
fn given_legacy_token_without_optional_metadata_when_decoding_then_component_is_required() -> TestResult {
    let record = Record::encode(&token_value()?)?;
    assert!(!token(ValueRef::Map(record.root()))?.optional);
    Ok(())
}

#[test]
fn given_boolean_optional_metadata_when_decoding_then_optional_component_is_preserved() -> TestResult {
    let record = with_optional(Value::Bool(true))?;
    assert!(token(ValueRef::Map(record.root()))?.optional);
    Ok(())
}

#[test]
fn given_non_boolean_optional_metadata_when_decoding_then_catalog_is_rejected() -> TestResult {
    for value in [Value::Null, Value::Int(1), Value::Text("true".to_owned())] {
        let record = with_optional(value)?;
        assert!(token(ValueRef::Map(record.root())).is_err());
    }
    Ok(())
}

#[test]
fn given_all_optional_tuple_when_decoding_then_catalog_is_rejected() -> TestResult {
    let component = with_optional(Value::Bool(true))?;
    let value = map([(1, map([(1, Value::Int(0))])?),
        (2, Value::List(vec![ValueRef::Map(component.root()).to_value()?]))])?;
    let record = Record::encode(&value)?;
    assert!(page(record.root()).is_err());
    Ok(())
}
