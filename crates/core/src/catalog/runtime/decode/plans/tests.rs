use super::*;
use crate::record::{FieldId, Key, MapRef, Record, Value};
use std::collections::BTreeMap;

fn map_bytes(
    entries: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Vec<u8>, crate::record::RecordError> {
    let fields = entries
        .into_iter()
        .map(|(field, value)| Ok((Key::Field(FieldId::new(field)?), value)))
        .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
    Ok(Record::encode(&Value::Map(fields))?.into_bytes())
}

#[test]
fn item_root_rejects_extra_fields() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([(1, Value::Int(0)), (9, Value::Int(2))])?;
    assert!(matches!(
        item_plan(MapRef::from_bytes_bounded(&bytes)?),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn identity_rejects_extra_fields() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([
        (1, Value::Int(0)),
        (2, Value::List(vec![Value::Int(1)])),
        (9, Value::Int(2)),
    ])?;
    assert!(matches!(
        identity(MapRef::from_bytes_bounded(&bytes)?),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn enumeration_purpose_rejects_region_fields() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?;
    assert!(matches!(
        purpose(MapRef::from_bytes_bounded(&bytes)?),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn account_binding_rejects_value_fields() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([
        (1, Value::List(vec![Value::Int(1)])),
        (2, Value::Int(0)),
        (3, Value::Text("extra".to_owned())),
    ])?;
    assert!(matches!(
        binding(ValueRef::Map(MapRef::from_bytes_bounded(&bytes)?)),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn null_region_status_is_absent() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([
        (1, Value::Int(2)),
        (2, Value::List(vec![Value::Int(1)])),
        (3, Value::List(vec![Value::Int(2)])),
        (4, Value::Null),
    ])?;
    assert!(matches!(
        purpose(MapRef::from_bytes_bounded(&bytes)?),
        Ok(PlanPurpose::RegionDiscovery {
            opt_in_status_path: None,
            ..
        })
    ));
    Ok(())
}
