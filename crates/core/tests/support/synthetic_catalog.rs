//! Test-only synthetic catalogs where the injected identity/pagination fixtures
//! have paths that actually resolve in the modeled shapes.

use inventory_core::{
    catalog::{CatalogKey, RuntimeCatalog, compiled_inputs_sha256},
    record::{FieldId, Key, ListRef, Record, Value, ValueRef, encode_list, encode_container},
};
use inventory_core::catalog::MAX_CATALOG_BYTES;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(entries.into_iter().map(|(id, value)| Ok((key(id)?, value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?))
}

fn string_field(name: &str, id: u32) -> TestResult<Value> {
    map(vec![
        (1, Value::Text(name.to_owned())),
        (2, Value::Int(id)),
        (3, Value::Int(2)), // kind = string
    ])
}

fn shape_structure(name: &str, id: u32, member_ids: &[u32]) -> TestResult<Value> {
    let members: Vec<Value> = member_ids.iter().map(|&member_id| {
        let field_name = format!("field{member_id}");
        string_field(&field_name, member_id)
    }).collect::<TestResult<Vec<_>>>()?;

    map(vec![
        (1, Value::Text(name.to_owned())),
        (2, Value::Int(id)),
        (3, Value::Int(14)), // kind = structure
        (4, Value::List(members)),
    ])
}

fn list_shape(name: &str, id: u32, element_id: u32) -> TestResult<Value> {
    map(vec![
        (1, Value::Text(name.to_owned())),
        (2, Value::Int(id)),
        (3, Value::Int(16)), // kind = list
        (4, Value::Int(element_id)),
    ])
}

fn field_registry_values() -> (Value, Value) {
    (
        Value::Bytes(b"input".to_vec()),
        Value::Bytes(vec![0, 0, 0, 0]),
    )
}

fn operation(input: u32, output: u32) -> Value {
    Value::Map(BTreeMap::from_iter([
        (key(1).unwrap(), Value::Text("DescribeVpcs".to_owned())),
        (key(2).unwrap(), Value::Int(0)),
        (key(3).unwrap(), Value::Int(input as i64)),
        (key(4).unwrap(), Value::Int(output as i64)),
    ]))
}

fn shape_offsets(shapes: &[Value]) -> TestResult<Value> {
    let bytes = encode_list(shapes)?;
    let offsets = ListRef::from_bytes(&bytes)?.encoded_item_offsets()?;
    Ok(Value::Bytes(
        offsets.iter().flat_map(|offset| offset.to_be_bytes()).collect(),
    ))
}

fn service(name: &str, operations: Vec<Value>, shapes: Vec<Value>) -> TestResult<Value> {
    let offsets = shape_offsets(&shapes)?;
    let mut fields = BTreeMap::new();
    fields.insert(key(1)?, Value::Text(name.to_owned()));
    fields.insert(key(2)?, Value::Text("1".to_owned()));
    fields.insert(key(3)?, Value::Text("n#Example".to_owned()));
    fields.insert(key(4)?, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())]));
    fields.insert(key(5)?, Value::List(operations));
    fields.insert(key(6)?, Value::List(shapes));
    fields.insert(key(7)?, offsets);
    fields.insert(key(8)?, Value::List(vec![]));
    Ok(Value::Map(fields))
}

fn root(service: Value) -> TestResult<Vec<u8>> {
    let (field_names, field_offsets) = field_registry_values();
    let mut fields = BTreeMap::new();
    fields.insert(key(1)?, Value::Int(1));
    fields.insert(key(2)?, Value::Text("test-commit".to_owned()));
    fields.insert(key(3)?, field_names);
    fields.insert(key(4)?, field_offsets);
    fields.insert(key(5)?, Value::List(vec![service]));
    encode_container(&Value::Map(fields), MAX_CATALOG_BYTES)
        .map_err(|e| e.into())
}

fn with_extension(bytes: &[u8], extension: Value) -> TestResult<Vec<u8>> {
    let mut root = ValueRef::Map(
        MapRef::from_container_bytes(bytes, MAX_CATALOG_BYTES)?,
    ).to_value()?;

    let fields = match root {
        Value::Map(ref mut map) => map,
        _ => return Err("root must be map".into()),
    };

    fields.insert(key(60_000)?, extension);
    Ok(encode_container(&root, MAX_CATALOG_BYTES).map_err(|e| e.into())?)
}

fn catalog_key_map(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    Ok(Value::Map(entries.into_iter().map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?))
}

fn shape_value(name: &str, id: i64, kind: i64, member_list: Value) -> Value {
    Value::Map(BTreeMap::from_iter([
        (Key::Field(FieldId::new(1).unwrap()), Value::Text(name.to_owned())),
        (Key::Field(FieldId::new(2).unwrap()), Value::Int(id)),
        (Key::Field(FieldId::new(3).unwrap()), Value::Int(kind)),
        (Key::Field(FieldId::new(4).unwrap()), member_list),
    ]))
}

fn member_value(id: i64, target: i64, name: &str) -> Value {
    Value::Map(BTreeMap::from_iter([
        (Key::Field(FieldId::new(1).unwrap()), Value::Int(id)),
        (Key::Field(FieldId::new(2).unwrap()), Value::Int(target)),
        (Key::Field(FieldId::new(3).unwrap()), Value::Bool(false)),
        (Key::Field(FieldId::new(4).unwrap()), Value::Text(name.to_owned())),
    ]))
}

pub fn scalar_item_catalog() -> TestResult<Vec<u8>> {
    // Single input shape with 3 string fields; identity uses empty path (scalar-self)
    let members = Value::List(vec![
        member_value(1, -1, "field1"),
        member_value(2, -1, "field2"),
        member_value(3, -1, "field3"),
    ]);
    let input = shape_value("DescribeVpcsInput", 1, 14, members);
    let output_members = Value::List(vec![member_value(1, -1, "Vpcs")]);
    let output = shape_value("DescribeVpcsOutput", 2, 14, output_members);
    let shapes = vec![input, output];
    let offsets = shape_offsets(&shapes)?;
    let operation = Value::Map(BTreeMap::from_iter([
        (key(1)?, Value::Text("DescribeVpcs".to_owned())),
        (key(2)?, Value::Int(0)),
        (key(3)?, Value::Int(1)),
        (key(4)?, Value::Int(2)),
    ]));
    let svc = catalog_key_map(vec![
        (CatalogKey::Name, Value::Text("ec2".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".to_owned())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, offsets),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    root(svc)
}

pub fn nested_structure_catalog(depth: u32) -> TestResult<Vec<u8>> {
    // Chain of nested structures for path traversal tests
    let mut shapes = Vec::new();
    let mut member_list = Value::List(vec![]);
    let base_id = 10;
    for level in 0..depth {
        let id = base_id + level as i64;
        let target_id = if level == depth - 1 { 1 } else { base_id + level as i64 + 1 };
        let name = format!("Level{id}");
        let member = member_value(1, target_id, &name);
        member_list = Value::List(vec![member]);
        let shape = shape_value(&name, id, 14, member_list.clone());
        shapes.push(shape);
    }
    let input_members = Value::List(vec![member_value(1, base_id, "root")]);
    let input = shape_value("DescribeVpcsInput", 1, 14, input_members);
    shapes.insert(0, input);
    let offsets = shape_offsets(&shapes)?;
    let operation = Value::Map(BTreeMap::from_iter([
        (key(1)?, Value::Text("DescribeVpcs".to_owned())),
        (key(2)?, Value::Int(0)),
        (key(3)?, Value::Int(1)),
        (key(4)?, Value::Int(-1)),
    ]));
    let svc = catalog_key_map(vec![
        (CatalogKey::Name, Value::Text("ec2".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".to_owned())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, offsets),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    root(svc)
}

pub fn pagination_catalog(input_field_ids: &[u32], output_field_ids: &[u32]) -> TestResult<Vec<u8>> {
    // Input shape with request token fields, output shape with response token fields
    let input_members: Vec<Value> = input_field_ids.iter().map(|&id| {
        member_value(i64::from(id), -1, &format!("input{ id }"))
    }).collect();
    let input = shape_value("DescribeVpcsInput", 1, 14, Value::List(input_members));

    let output_members: Vec<Value> = output_field_ids.iter().map(|&id| {
        member_value(i64::from(id), -1, &format!("output{ id }"))
    }).collect();
    let output = shape_value("DescribeVpcsOutput", 2, 14, Value::List(output_members));

    let shapes = vec![input, output];
    let offsets = shape_offsets(&shapes)?;
    let operation = Value::Map(BTreeMap::from_iter([
        (key(1)?, Value::Text("DescribeVpcs".to_owned())),
        (key(2)?, Value::Int(0)),
        (key(3)?, Value::Int(1)),
        (key(4)?, Value::Int(2)),
    ]));
    let svc = catalog_key_map(vec![
        (CatalogKey::Name, Value::Text("ec2".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".to_owned())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, offsets),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    root(svc)
}

pub fn identity_catalog(field_ids: &[u32]) -> TestResult<Vec<u8>> {
    // Input shape with fields for identity path tests
    let input_members: Vec<Value> = field_ids.iter().map(|&id| {
        member_value(i64::from(id), -1, &format!("field{ id }"))
    }).collect();
    let input = shape_value("DescribeVpcsInput", 1, 14, Value::List(input_members));
    let output = shape_value("DescribeVpcsOutput", 2, 14, Value::List(vec![]));
    let shapes = vec![input, output];
    let offsets = shape_offsets(&shapes)?;
    let operation = Value::Map(BTreeMap::from_iter([
        (key(1)?, Value::Text("DescribeVpcs".to_owned())),
        (key(2)?, Value::Int(0)),
        (key(3)?, Value::Int(1)),
        (key(4)?, Value::Int(2)),
    ]));
    let svc = catalog_key_map(vec![
        (CatalogKey::Name, Value::Text("ec2".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::ServiceShape, Value::Text("n#Example".to_owned())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, offsets),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    root(svc)
}