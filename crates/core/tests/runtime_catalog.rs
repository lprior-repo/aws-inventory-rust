use inventory_core::{
    catalog::runtime::{RelationScope, ReviewDisposition, RuntimeCatalog, RuntimeCatalogError},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use std::collections::BTreeMap;

const BASE: &[u8] = include_bytes!("../../../catalog/aws-api.catalog");

fn field_map(
    entries: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let entries = entries
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Value::Map(entries))
}

fn runtime_bytes(version: i64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(version)),
        (2, Value::List(Vec::new())),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(Vec::new())),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    Ok(Record::encode(&Value::Map(root.clone()))?.into_bytes())
}

fn schema_value(kind: &str, table: &str) -> Result<Value, Box<dyn std::error::Error>> {
    field_map([
        (1, Value::Text(kind.to_owned())),
        (2, Value::Text(table.to_owned())),
        (3, Value::List(Vec::new())),
        (4, Value::Null),
    ])
}

fn coverage_value(kind: &str) -> Result<Value, Box<dyn std::error::Error>> {
    field_map([
        (1, Value::Text(kind.to_owned())),
        (2, Value::Text(kind.to_owned())),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(false)),
    ])
}

fn denied_runtime_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let catalog = inventory_core::catalog::Catalog::borrow(BASE)?;
    let service = catalog
        .services()?
        .next()
        .ok_or("base catalog has no service")??;
    let operation = service
        .operations()?
        .next()
        .ok_or("base service has no operation")??;
    let decision = field_map([
        (1, Value::Text(service.name()?.to_owned())),
        (2, Value::Text(operation.name()?.to_owned())),
        (3, Value::Int(1)),
        (4, Value::Text("explicitly reviewed denial".to_owned())),
    ])?;
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(1)),
        (2, Value::List(Vec::new())),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(vec![decision])),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    Ok(Record::encode(&Value::Map(root.clone()))?.into_bytes())
}

fn relationship_runtime_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let source_kind = "AWS::EC2::Subnet";
    let target_kind = "AWS::EC2::VPC";
    let relation = field_map([
        (1, Value::Text(source_kind.to_owned())),
        (2, Value::List(vec![Value::Int(1)])),
        (3, Value::Text(target_kind.to_owned())),
        (4, Value::List(vec![Value::Int(2)])),
        (5, Value::Text("IN_VPC".to_owned())),
        (6, Value::Int(2)),
    ])?;
    let extension = field_map([
        (1, Value::Int(1)),
        (2, Value::List(Vec::new())),
        (
            3,
            Value::List(vec![
                schema_value(source_kind, "aws_ec2_subnet")?,
                schema_value(target_kind, "aws_ec2_vpc")?,
            ]),
        ),
        (4, Value::List(vec![relation])),
        (
            5,
            Value::List(vec![
                coverage_value(source_kind)?,
                coverage_value(target_kind)?,
            ]),
        ),
        (6, Value::List(Vec::new())),
    ])?;
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    Ok(Record::encode(&Value::Map(root.clone()))?.into_bytes())
}

#[test]
fn actual_compiled_catalog_loads_exactly_the_reviewed_callable_plans()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = RuntimeCatalog::from_bytes(BASE)?;
    let mut plans = runtime
        .plans()
        .iter()
        .map(|plan| (plan.service_name(), plan.operation_name()))
        .collect::<Vec<_>>();
    plans.sort_unstable();
    assert_eq!(
        plans,
        [
            ("ec2", "DescribeRegions"),
            ("ec2", "DescribeSubnets"),
            ("ec2", "DescribeVpcs"),
            ("sts", "GetCallerIdentity")
        ]
    );
    Ok(())
}

#[test]
fn unknown_runtime_extension_version_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(2)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::ExtensionVersion(2))
    ));
    Ok(())
}

#[test]
fn valid_empty_runtime_extension_keeps_borrowed_catalog_and_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = runtime_bytes(1)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(runtime.plans().is_empty());
    assert!(runtime.schemas().is_empty());
    assert!(runtime.decisions().is_empty());
    assert_eq!(runtime.raw_bytes(), bytes.as_slice());
    Ok(())
}

#[test]
fn incomplete_plan_metadata_is_rejected_as_a_closed_capability()
-> Result<(), Box<dyn std::error::Error>> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
    let root = match &mut root {
        Value::Map(fields) => fields,
        _ => return Err("base compiled catalog is not a map".into()),
    };
    let extension = field_map([
        (1, Value::Int(1)),
        (2, Value::List(vec![field_map([])?])),
        (3, Value::List(Vec::new())),
        (4, Value::List(Vec::new())),
        (5, Value::List(Vec::new())),
        (6, Value::List(Vec::new())),
    ])?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    let bytes = Record::encode(&Value::Map(root.clone()))?.into_bytes();
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn denied_review_decision_is_retained_without_minting_a_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = denied_runtime_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(runtime.plans().is_empty());
    assert_eq!(runtime.decisions().len(), 1);
    assert_eq!(
        runtime.decisions()[0].disposition,
        ReviewDisposition::Denied {
            reason: "explicitly reviewed denial".to_owned(),
        }
    );
    Ok(())
}

#[test]
fn relationship_scope_preserves_home_region_as_distinct_typed_value()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = relationship_runtime_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(runtime.relationships().len(), 1);
    assert_eq!(runtime.relationships()[0].scope, RelationScope::HomeRegion);
    Ok(())
}
