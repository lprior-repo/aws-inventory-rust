use super::*;
use crate::{catalog::runtime::RelationSource, record::{Key, Record, Value}};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn encoded_relation(source: Option<Value>) -> Result<Record, Box<dyn std::error::Error>> {
    let fields = [(1, Value::Text("AWS::EC2::Subnet".to_owned())),
        (2, Value::List(vec![Value::Int(1)])),
        (3, Value::Text("AWS::EC2::VPC".to_owned())),
        (4, Value::List(vec![Value::Int(2)])),
        (5, Value::Text("IN_VPC".to_owned())), (6, Value::Int(0))];
    let mut fields = fields.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
    if let Some(source) = source { fields.insert(Key::Field(FieldId::new(10)?), source); }
    Ok(Record::encode(&Value::Map(fields))?)
}

#[test]
fn relation_source_decodes_explicit_namespaces_and_legacy_selected_default() -> TestResult {
    for (wire, expected) in [(None, RelationSource::SelectedOutput),
        (Some(Value::Int(0)), RelationSource::SelectedOutput),
        (Some(Value::Int(1)), RelationSource::Request)] {
        let encoded = encoded_relation(wire)?;
        let decoded = relation(ValueRef::Map(encoded.root()))?;
        assert_eq!(decoded.source, expected);
        assert_eq!(decoded.source_path.as_ref(), &[FieldId::new(1)?]);
    }
    Ok(())
}

#[test]
fn relation_source_rejects_unknown_and_wrongly_typed_wire_values() -> TestResult {
    for wire in [Value::Int(-1), Value::Int(2), Value::Text("Request".to_owned()), Value::Null, Value::Bool(true)] {
        let encoded = encoded_relation(Some(wire))?;
        assert!(matches!(relation(ValueRef::Map(encoded.root())), Err(RuntimeCatalogError::Capability)));
    }
    Ok(())
}
