use super::*;
use crate::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fields(values: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        values
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?,
    ))
}
fn alternative(name: &str, source: i64) -> TestResult<Value> {
    fields([
        (1, Value::Text("binding-test".into())),
        (2, Value::Text(name.into())),
        (3, Value::Int(source)),
        (4, Value::List(vec![Value::Int(7)])),
    ])
}
fn binding(entries: Vec<Value>) -> TestResult<Value> {
    fields([
        (1, Value::List(vec![Value::Int(9)])),
        (2, Value::Int(12)),
        (3, Value::List(entries)),
    ])
}
fn parse(
    value: &Value,
) -> TestResult<Result<crate::catalog::runtime::BindingPlan, RuntimeCatalogError>> {
    let record = Record::encode(value)?;
    Ok(super::super::binding(ValueRef::Map(record.root())))
}

#[test]
fn given_one_or_eight_producers_when_decoded_then_all_namespaces_paths_and_order_are_exact()
-> TestResult {
    for count in [1, 8] {
        let entries = (0..count)
            .map(|index| alternative(&format!("ListParents{index}"), index % 3))
            .collect::<TestResult<Vec<_>>>()?;
        let decoded = parse(&binding(entries)?)??;
        assert_eq!(decoded.path.as_ref(), &[FieldId::new(9)?]);
        let BindingSource::Alternatives(alternatives) = decoded.source else {
            return Err("wrong binding tag".into());
        };
        assert_eq!(alternatives.len(), usize::try_from(count)?);
        for (index, alternative) in alternatives.iter().enumerate() {
            assert_eq!(alternative.service.as_ref(), "binding-test");
            assert_eq!(
                alternative.operation.as_ref(),
                format!("ListParents{index}")
            );
            assert_eq!(alternative.path.as_ref(), &[FieldId::new(7)?]);
            assert_eq!(
                [
                    ContextSource::SelectedItem,
                    ContextSource::Response,
                    ContextSource::Request
                ]
                .get(index % 3),
                Some(&alternative.source)
            );
        }
    }
    Ok(())
}

#[test]
fn given_empty_duplicate_or_nine_alternatives_when_decoded_then_metadata_fails_closed() -> TestResult
{
    assert!(matches!(
        parse(&binding(Vec::new())?)?,
        Err(RuntimeCatalogError::Capability)
    ));
    assert!(matches!(
        parse(&binding(vec![
            alternative("ListA", 0)?,
            alternative("ListA", 2)?
        ])?)?,
        Err(RuntimeCatalogError::Capability)
    ));
    let excessive = (0..9)
        .map(|index| alternative(&format!("List{index}"), 0))
        .collect::<TestResult<Vec<_>>>()?;
    assert!(matches!(
        parse(&binding(excessive)?)?,
        Err(RuntimeCatalogError::Bounds)
    ));
    Ok(())
}

#[test]
fn given_corrupted_alternative_fields_when_decoded_then_no_namespace_or_path_is_guessed()
-> TestResult {
    for corruption in 0..8 {
        let mut entry = alternative("ListA", 0)?;
        let Value::Map(values) = &mut entry else {
            return Err("entry is not map".into());
        };
        match corruption {
            0 => {
                values.insert(Key::Field(FieldId::new(3)?), Value::Int(3));
            }
            1 => {
                values.insert(Key::Field(FieldId::new(3)?), Value::Text("Request".into()));
            }
            2 => {
                values.insert(
                    Key::Field(FieldId::new(4)?),
                    Value::List(vec![Value::Int(0)]),
                );
            }
            3 => {
                values.insert(
                    Key::Field(FieldId::new(4)?),
                    Value::List(vec![Value::Int(1); 9]),
                );
            }
            4 => {
                values.remove(&Key::Field(FieldId::new(1)?));
            }
            5 => {
                values.remove(&Key::Field(FieldId::new(2)?));
            }
            6 => {
                values.insert(Key::Field(FieldId::new(5)?), Value::Null);
            }
            _ => {
                values.insert(Key::Name("source".into()), Value::Int(0));
            }
        }
        assert!(
            parse(&binding(vec![entry])?)?.is_err(),
            "accepted corruption {corruption}"
        );
    }
    Ok(())
}

#[test]
fn given_region_list_when_decoded_then_only_its_typed_tag_and_destination_are_accepted()
-> TestResult {
    let value = fields([(1, Value::List(vec![Value::Int(9)])), (2, Value::Int(11))])?;
    let decoded = parse(&value)??;
    assert!(matches!(decoded.source, BindingSource::RegionList));
    let Value::Map(mut values) = value else {
        return Err("binding not map".into());
    };
    values.insert(
        Key::Field(FieldId::new(3)?),
        Value::Text("us-east-1".into()),
    );
    assert!(matches!(
        parse(&Value::Map(values))?,
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
