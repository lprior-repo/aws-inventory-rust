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
fn emit_aware_reader_rejects_skip_extension_field_26() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([(25, Value::Int(1)), (26, Value::List(Vec::new()))])?;
    let fields = MapRef::from_bytes_bounded(&bytes)?;
    assert!(matches!(
        metadata::only_fields(fields, &(1..=25).collect::<Vec<_>>()),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
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

fn field_map(entries: &[(u32, Value)]) -> Result<Value, crate::record::RecordError> {
    let fields = entries
        .iter()
        .map(|(field, value)| Ok((Key::Field(FieldId::new(*field)?), value.clone())))
        .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
    Ok(Value::Map(fields))
}

fn exception(
    region: &str,
    url: &str,
    signing_name: &str,
    signing_region: &str,
) -> Result<Value, crate::record::RecordError> {
    field_map(&[
        (1, Value::Text(region.to_owned())),
        (2, Value::Text(url.to_owned())),
        (3, Value::Text(signing_name.to_owned())),
        (4, Value::Text(signing_region.to_owned())),
    ])
}

fn regional_endpoint(exceptions: Vec<Value>) -> Result<Vec<u8>, crate::record::RecordError> {
    map_bytes([
        (
            1,
            Value::Text("https://svc.{region}.example.com".to_owned()),
        ),
        (2, Value::Text("{region}".to_owned())),
        (3, Value::Text("svc".to_owned())),
        (4, Value::Int(1)),
        (5, Value::Int(0)),
        (6, Value::List(exceptions)),
    ])
}

#[test]
fn endpoint_exception_overrides_only_its_region() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = regional_endpoint(vec![exception(
        "eu-west-1",
        "https://svc-fips.eu-west-1.example.com",
        "svc-eu",
        "eu-west-1",
    )?])?;
    let template = endpoint(MapRef::from_bytes_bounded(&bytes)?)?;
    let eu_region = crate::inventory::RegionId::parse("eu-west-1")?;
    let eu = template.resolve(
        &eu_region,
        &crate::inventory::Region::Regional(eu_region.clone()),
    )?;
    assert_eq!(eu.url, "https://svc-fips.eu-west-1.example.com");
    assert_eq!(eu.signing_name, "svc-eu");
    assert_eq!(eu.signing_region, "eu-west-1");
    let us_region = crate::inventory::RegionId::parse("us-east-1")?;
    let us = template.resolve(
        &us_region,
        &crate::inventory::Region::Regional(us_region.clone()),
    )?;
    assert_eq!(us.url, "https://svc.us-east-1.example.com");
    assert_eq!(us.signing_name, "svc");
    assert_eq!(us.signing_region, "us-east-1");
    Ok(())
}

fn global_endpoint(exceptions: Vec<Value>) -> Result<Vec<u8>, crate::record::RecordError> {
    map_bytes([
        (1, Value::Text("https://svc.example.com".to_owned())),
        (2, Value::Text("us-east-1".to_owned())),
        (3, Value::Text("svc".to_owned())),
        (4, Value::Int(0)),
        (5, Value::Int(0)),
        (6, Value::List(exceptions)),
    ])
}

fn global_pseudo_region_exception(label: &str) -> Result<Value, crate::record::RecordError> {
    exception(
        label,
        "https://global-override.example.com",
        "svc-global",
        "us-east-1",
    )
}

#[test]
fn endpoint_exception_for_global_pseudo_region_applies_at_global_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = global_endpoint(vec![global_pseudo_region_exception("-")?])?;
    let home = crate::inventory::RegionId::parse("us-east-1")?;
    let resolved = endpoint(MapRef::from_bytes_bounded(&bytes)?)?
        .resolve(&home, &crate::inventory::Region::Global)?;
    assert_eq!(resolved.url, "https://global-override.example.com");
    assert_eq!(resolved.signing_name, "svc-global");
    Ok(())
}

#[test]
fn endpoint_exception_named_aws_global_applies_at_global_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = global_endpoint(vec![global_pseudo_region_exception("aws-global")?])?;
    let home = crate::inventory::RegionId::parse("us-east-1")?;
    let resolved = endpoint(MapRef::from_bytes_bounded(&bytes)?)?
        .resolve(&home, &crate::inventory::Region::Global)?;
    assert_eq!(resolved.url, "https://global-override.example.com");
    assert_eq!(resolved.signing_name, "svc-global");
    Ok(())
}

#[test]
fn global_pseudo_region_exception_ignored_under_regional_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = global_endpoint(vec![global_pseudo_region_exception("-")?])?;
    let home = crate::inventory::RegionId::parse("us-east-1")?;
    let resolved = endpoint(MapRef::from_bytes_bounded(&bytes)?)?
        .resolve(&home, &crate::inventory::Region::Regional(home.clone()))?;
    assert_eq!(resolved.url, "https://svc.example.com");
    assert_eq!(resolved.signing_name, "svc");
    Ok(())
}

#[test]
fn regional_endpoint_ignores_global_pseudo_region_exception()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = regional_endpoint(vec![global_pseudo_region_exception("-")?])?;
    let home = crate::inventory::RegionId::parse("us-east-1")?;
    let resolved = endpoint(MapRef::from_bytes_bounded(&bytes)?)?
        .resolve(&home, &crate::inventory::Region::Global)?;
    assert_eq!(resolved.url, "https://svc.us-east-1.example.com");
    assert_eq!(resolved.signing_name, "svc");
    Ok(())
}

#[test]
fn home_region_exception_beats_global_pseudo_region_exception()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = global_endpoint(vec![
        global_pseudo_region_exception("-")?,
        exception(
            "us-east-1",
            "https://home-override.example.com",
            "svc-home",
            "us-east-1",
        )?,
    ])?;
    let home = crate::inventory::RegionId::parse("us-east-1")?;
    let resolved = endpoint(MapRef::from_bytes_bounded(&bytes)?)?
        .resolve(&home, &crate::inventory::Region::Global)?;
    assert_eq!(resolved.url, "https://home-override.example.com");
    assert_eq!(resolved.signing_name, "svc-home");
    Ok(())
}

#[test]
fn endpoint_rejects_unknown_field_seven() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = map_bytes([
        (1, Value::Text("https://svc.example.com".to_owned())),
        (2, Value::Text("us-east-1".to_owned())),
        (3, Value::Text("svc".to_owned())),
        (4, Value::Int(0)),
        (5, Value::Int(0)),
        (7, Value::Int(0)),
    ])?;
    assert!(matches!(
        endpoint(MapRef::from_bytes_bounded(&bytes)?),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn endpoint_rejects_duplicate_exception_regions() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = regional_endpoint(vec![
        exception("eu-west-1", "https://a.example.com", "svc", "eu-west-1")?,
        exception("eu-west-1", "https://b.example.com", "svc", "eu-west-1")?,
    ])?;
    assert!(matches!(
        endpoint(MapRef::from_bytes_bounded(&bytes)?),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
