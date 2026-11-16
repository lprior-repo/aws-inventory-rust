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
    for wire in [Value::Int(-1), Value::Int(3), Value::Text("Request".to_owned()), Value::Null, Value::Bool(true)] {
        let encoded = encoded_relation(Some(wire))?;
        assert!(matches!(relation(ValueRef::Map(encoded.root())), Err(RuntimeCatalogError::Capability)));
    }
    Ok(())
}

fn typed_relation(fields: impl IntoIterator<Item = (u32, Value)>) -> Result<Record, Box<dyn std::error::Error>> {
    let record = encoded_relation(None)?;
    let Value::Map(mut values) = ValueRef::Map(record.root()).to_value()? else { return Err("relation fixture is not a map".into()); };
    for (field, value) in fields { values.insert(Key::Field(FieldId::new(field)?), value); }
    Ok(Record::encode(&Value::Map(values))?)
}

#[test]
fn typed_relation_modes_guard_parent_and_compound_survive_decode() -> TestResult {
    use crate::catalog::runtime::{RelationPathMode, RelationTargetMode};
    // Given explicit root source and canonical target, guarded by a real request field.
    let guard = Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(1)),
        (Key::Field(FieldId::new(2)?), Value::Int(0)),
        (Key::Field(FieldId::new(3)?), Value::List(vec![Value::Int(3)])),
        (Key::Field(FieldId::new(4)?), Value::Text("active".into())),
    ]));
    let encoded = typed_relation([(2, Value::List(Vec::new())), (4, Value::List(Vec::new())),
        (11, Value::Int(1)), (12, Value::Int(4)), (13, guard)])?;
    // When decoding the actual numeric extension row.
    let decoded = relation(ValueRef::Map(encoded.root()))?;
    // Then neither a sentinel field nor a dropped guard can masquerade as the typed rule.
    assert_eq!(decoded.source_path_mode, RelationPathMode::RootText);
    assert_eq!(decoded.target_mode, RelationTargetMode::CanonicalId);
    let guard = decoded.guard.ok_or("typed guard was dropped")?;
    assert_eq!(guard.source, RelationSource::Request);
    assert_eq!(guard.path.as_ref(), &[FieldId::new(3)?]);
    assert_eq!(guard.equals, "active");

    let encoded = typed_relation([(2, Value::List(Vec::new())), (4, Value::List(Vec::new())),
        (10, Value::Int(2)), (11, Value::Int(1)), (12, Value::Int(4))])?;
    assert_eq!(relation(ValueRef::Map(encoded.root()))?.source, RelationSource::Parent);

    let component = Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::Int(1)),
        (Key::Field(FieldId::new(2)?), Value::List(vec![Value::Int(3)])),
        (Key::Field(FieldId::new(3)?), Value::Int(0)),
        (Key::Field(FieldId::new(4)?), Value::List(vec![Value::Int(4)])),
        (Key::Field(FieldId::new(5)?), Value::Int(2)),
    ]));
    let encoded = typed_relation([(14, Value::List(vec![component.clone()]))])?;
    let decoded = relation(ValueRef::Map(encoded.root()))?;
    let term = decoded.components.first().ok_or("AND term was dropped")?;
    assert_eq!(term.source, RelationSource::Request);
    assert_eq!(term.target_mode, RelationTargetMode::BoundContextFields);
    assert_eq!(term.target_path.as_ref(), &[FieldId::new(4)?]);
    let oversized = typed_relation([(14, Value::List(vec![component; 8]))])?;
    assert!(matches!(relation(ValueRef::Map(oversized.root())), Err(RuntimeCatalogError::Bounds)));
    Ok(())
}

#[test]
fn typed_relation_rejects_mode_path_mismatches_and_parent_field_selection() -> TestResult {
    // Given modes that disagree with the modeled field paths.
    for fields in [vec![(11, Value::Int(1))], vec![(12, Value::Int(4))],
        vec![(10, Value::Int(2)), (4, Value::List(Vec::new())), (12, Value::Int(4))],
        vec![(11, Value::Int(2))], vec![(12, Value::Int(5))]] {
        let encoded = typed_relation(fields)?;
        // When decoding, no invalid namespace is downcast to a legacy field selector.
        assert!(matches!(relation(ValueRef::Map(encoded.root())), Err(RuntimeCatalogError::Capability)));
    }
    Ok(())
}
