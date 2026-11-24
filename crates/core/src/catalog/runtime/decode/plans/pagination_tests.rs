use super::*;
use crate::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn map(
    entries: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Value, crate::record::RecordError> {
    entries
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map(Value::Map)
}

fn token_value() -> Result<Value, crate::record::RecordError> {
    let path = Value::List(vec![Value::Int(1)]);
    let placement = map([(1, Value::Int(3)), (2, path.clone())])?;
    map([
        (1, path.clone()),
        (2, path),
        (3, Value::Int(0)),
        (4, placement.clone()),
        (5, placement),
    ])
}

fn with_optional(value: Value) -> Result<Record, crate::record::RecordError> {
    let Value::Map(mut fields) = token_value()? else {
        return Err(crate::record::RecordError::NotMap);
    };
    fields.insert(Key::Field(FieldId::new(6)?), value);
    Record::encode(&Value::Map(fields))
}

#[test]
fn given_legacy_token_without_optional_metadata_when_decoding_then_component_is_required()
-> TestResult {
    let record = Record::encode(&token_value()?)?;
    assert!(!token(ValueRef::Map(record.root()))?.optional);
    Ok(())
}

#[test]
fn given_boolean_optional_metadata_when_decoding_then_optional_component_is_preserved() -> TestResult
{
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
    let value = map([
        (1, map([(1, Value::Int(0))])?),
        (
            2,
            Value::List(vec![ValueRef::Map(component.root()).to_value()?]),
        ),
    ])?;
    let record = Record::encode(&value)?;
    assert!(page(record.root()).is_err());
    Ok(())
}

fn page_with_size(
    value_type: Option<Value>,
    maximum: i64,
) -> Result<Record, crate::record::RecordError> {
    let path = Value::List(vec![Value::Int(1)]);
    let placement = map([(1, Value::Int(3)), (2, path.clone())])?;
    let size = map([(1, path), (2, Value::Int(maximum)), (3, placement)]
        .into_iter()
        .chain(value_type.map(|kind| (4, kind))))?;
    Record::encode(&map([
        (1, map([(1, Value::Int(0))])?),
        (2, Value::List(Vec::new())),
        (3, size),
    ])?)
}

#[test]
fn given_typed_page_limits_when_decoded_then_only_string_and_integer_are_serializable() -> TestResult
{
    use crate::catalog::runtime::PageSizeType;
    for (kind, expected, literal) in [
        (None, PageSizeType::Integer, Value::Int(100)),
        (Some(Value::Int(1)), PageSizeType::Integer, Value::Int(100)),
        (
            Some(Value::Int(0)),
            PageSizeType::String,
            Value::Text("100".into()),
        ),
    ] {
        let record = page_with_size(kind, 100)?;
        let page = page(record.root())?;
        let size = page.page_size.ok_or("page size")?;
        assert_eq!(size.value_type, expected);
        assert_eq!(size.value(), literal);
    }
    for kind in [
        Value::Int(-1),
        Value::Int(2),
        Value::Int(3),
        Value::Null,
        Value::Bool(true),
    ] {
        assert!(page(page_with_size(Some(kind), 100)?.root()).is_err());
    }
    for maximum in [-1, 0, i64::MAX] {
        assert!(page(page_with_size(Some(Value::Int(0)), maximum)?.root()).is_err());
    }
    Ok(())
}

#[test]
fn given_a_cap_when_the_page_size_is_built_then_it_is_the_smaller_of_cap_and_model_maximum()
-> TestResult {
    // Given one integer and one string page-size member, each with a model maximum of 100.
    let integer = page(page_with_size(Some(Value::Int(1)), 100)?.root())?
        .page_size
        .ok_or("integer page size")?;
    let text = page(page_with_size(Some(Value::Int(0)), 100)?.root())?
        .page_size
        .ok_or("string page size")?;
    // When no cap, a cap below, at, just above and far above the maximum is applied.
    for (cap, expected) in [
        (None, 100_i64),
        (Some(1), 1),
        (Some(99), 99),
        (Some(100), 100),
        (Some(101), 100),
        (Some(u32::MAX), 100),
    ] {
        // Then the value is never above the model maximum and keeps its wire type.
        assert_eq!(integer.capped(cap), Value::Int(expected));
        assert_eq!(text.capped(cap), Value::Text(expected.to_string()));
    }
    Ok(())
}
