use inventory_core::{
    catalog::{
        runtime::{RuntimeCatalog, compiled_inputs_sha256},
        CatalogKey, MAX_CATALOG_BYTES,
    },
    record::{FieldId, Key, MapRef, Record, Value, encode_list},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;

fn key(id: u32) -> Result<Key, RecordError> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut Fields> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("synthetic catalog fixture expected a map".into()),
    }
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(entries.into_iter().map(|(id, value)| Ok((key(id)?, value)))
        .collect::<Result<Fields, RecordError>>()?))
}

fn catalog_key_map(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    Ok(Value::Map(entries.into_iter().map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<Result<Fields, RecordError>>()?))
}

fn shape_offsets(shapes: &[Value]) -> Result<Value, RecordError> {
    let bytes = encode_list(shapes)?;
    let offsets = inventory_core::record::ListRef::from_bytes(&bytes)?.encoded_item_offsets()?;
    Ok(Value::Bytes(
        offsets.iter().flat_map(|offset| offset.to_be_bytes()).collect(),
    ))
}

fn field_name_bytes(names: &[&str]) -> (Value, Value) {
    let data = names.join("\0");
    let mut offsets = Vec::new();
    let mut position = 0;
    for &name in names {
        offsets.extend_from_slice(&position.to_be_bytes());
        position += name.len();
    }
    offsets.extend_from_slice(&position.to_be_bytes());
    (Value::Bytes(data.into_bytes()), Value::Bytes(offsets))
}

fn structure_shape(name: &str, members: &[Value]) -> TestResult<Value> {
    let field_name = format!("n#{name}");
    let (names, offsets) = field_name_bytes(&[name]);
    Ok(catalog_key_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(members.to_vec())),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
    ])?))
}

fn structure_member(id: u32, target: i64, required: bool) -> TestResult<Value> {
    Ok(catalog_key_map([
        (CatalogKey::MemberId, Value::Int(i64::from(id))),
        (CatalogKey::MemberTarget, Value::Int(target)),
        (CatalogKey::MemberRequired, Value::Bool(required)),
    ])?))
}

fn string_shape(name: &str) -> TestResult<Value> {
    let field_name = format!("n#{name}");
    let (names, offsets) = field_name_bytes(&[name]);
    Ok(catalog_key_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::Kind, Value::Int(2)),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
    ])?))
}

fn list_shape(name: &str, member_id: u32, element_target: i64) -> TestResult<Value> {
    let field_name = format!("n#{name}");
    let (names, offsets) = field_name_bytes(&[name]);
    let member = structure_member(member_id, element_target, false)?;
    Ok(catalog_key_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::Kind, Value::Int(17)),
        (CatalogKey::Members, Value::List(vec![member])),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
    ])?))
}

fn operation(name: &str, input: i64, output: i64) -> TestResult<Value> {
    Ok(catalog_key_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(input)),
        (CatalogKey::Output, Value::Int(output)),
    ])?))
}

fn service(name: &str, shapes: &[Value], operations: &[Value]) -> TestResult<Value> {
    let field_name = format!("n#{name}");
    let shape_offsets_bytes = shape_offsets(shapes)?;
    Ok(catalog_key_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (CatalogKey::ServiceShape, Value::Text(field_name)),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())])),
        (CatalogKey::Operations, Value::List(operations.to_vec())),
        (CatalogKey::Shapes, Value::List(shapes.to_vec())),
        (CatalogKey::ShapeOffsets, shape_offsets_bytes),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?))
}

fn root(service: Value, runtime_extension: Value) -> TestResult<Vec<u8>> {
    let service_bytes = Record::encode(&service)?.into_bytes();
    let (field_names, field_offsets) = field_name_bytes(&[]);
    let root = catalog_key_map([
        (CatalogKey::Format, Value::Int(1)),
        (CatalogKey::ModelsCommit, Value::Text("test-commit".to_owned())),
        (CatalogKey::FieldNames, field_names),
        (CatalogKey::FieldNameOffsets, field_offsets),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(service_bytes)])),
        (CatalogKey::RuntimeExtension, runtime_extension),
    ])?;
    Ok(Record::encode(&root)?.into_bytes())
}

fn plan(service: &str, operation: &str, kind: &str, purpose: i64, scope: i64,
    item: Value, identity: Value, pagination: Value) -> TestResult<Value> {
    Ok(map([
        (1, Value::Text(service.to_owned())),
        (2, Value::Text(operation.to_owned())),
        (3, Value::Text(kind.to_owned())),
        (4, Value::Int(purpose)),
        (5, Value::Int(scope)),
        (6, identity),
        (7, item),
        (8, Value::Int(0)),
        (9, pagination),
        (10, Value::List(vec![])),
        (11, Value::List(vec![])),
        (12, Value::List(vec![])),
        (13, Value::List(vec![])),
    ])?))
}

fn identity_plan(kind: i64, parts: Vec<Value>) -> TestResult<Value> {
    map([(1, Value::Int(kind)), (2, Value::List(parts))])
}

fn item_select(path: Vec<Value>) -> TestResult<Value> {
    map([(1, Value::Int(2)), (2, Value::List(path))])
}

fn pagination_plan(tokens: Vec<Value>) -> TestResult<Value> {
    map([(1, Value::List(tokens))])
}

fn pagination_token(request: Vec<Value>, response: Vec<Value>, optional: bool, value_type: i64) -> TestResult<Value> {
    map([
        (1, Value::List(request)),
        (2, Value::List(response)),
        (3, Value::Bool(optional)),
        (4, Value::Int(value_type)),
        (5, Value::Int(1)),
        (6, Value::Int(1)),
    ])
}

fn runtime_extension(plans: Vec<Value>) -> TestResult<Value> {
    Ok(Value::Map([
        (key(1)?, Value::Int(2)),
        (key(2)?, Value::List(plans)),
        (key(3)?, Value::List(vec![])),
        (key(4)?, Value::List(vec![])),
        (key(5)?, Value::List(vec![])),
        (key(6)?, Value::List(vec![])),
    ].into_iter().collect()))
}

/// Build a synthetic catalog where the item plan resolves to a scalar string field.
/// Supports tests that require scalar-self identity parts (empty paths).
pub fn scalar_item_catalog(identity: Value, pagination: Value) -> TestResult<Vec<u8>> {
    let service_name = "TestService";
    let operation_name = "ListThings";

    let input_shape = structure_shape("Input", &[])?;
    let item_string = string_shape("ItemString")?;
    let items_list = list_shape("ItemList", 1, -(i64::try_from(2)?) - 1)?;
    let output_shape = structure_shape("Output", &[structure_member(1, -(i64::try_from(3)?) - 1, false)?])?;

    let shapes = vec![input_shape, item_string, items_list, output_shape];
    let op = operation(operation_name, -(i64::try_from(1)?) - 1, -(i64::try_from(4)?) - 1)?;
    let svc = service(service_name, &shapes, &[op])?;

    let item = item_select(vec![Value::Int(1), Value::Int(1)])?;
    let extension = runtime_extension(vec![
        plan(service_name, operation_name, "vpc", 1, 1, item, identity, pagination)?
    ])?;

    let mut root = root(svc, extension)?;
    Ok(root)
}

/// Build a synthetic catalog with a deeply nested structure supporting N-segment paths.
/// Each level has a single member (field ID 1) that resolves to the next structure.
/// The innermost structure has a scalar string member.
pub fn nested_structure_catalog(depth: u32, identity: Value, pagination: Value) -> TestResult<Vec<u8>> {
    let service_name = "TestService";
    let operation_name = "ListThings";

    let mut shapes = Vec::new();
    let scalar = string_shape("Scalar")?;
    shapes.push(scalar);

    for i in 0..depth {
        let name = format!("Level{i}");
        let member = structure_member(1, -(i64::try_from(shapes.len() + 1)?) - 1, false)?;
        let shape = structure_shape(&name, &[member])?;
        shapes.push(shape);
    }

    let items_list = list_shape("ItemList", 1, -(i64::try_from(depth + 1)?) - 1)?;
    shapes.push(items_list);

    let input_shape = structure_shape("Input", &[])?;
    shapes.push(input_shape);
    let input_index = shapes.len() - 1;

    let output_shape = structure_shape("Output", &[structure_member(1, -(i64::try_from(depth + 1)?) - 1, false)?])?;
    shapes.push(output_shape);
    let output_index = shapes.len() - 1;

    let op = operation(operation_name, -(i64::try_from(input_index + 1)?) - 1, -(i64::try_from(output_index + 1)?) - 1)?;
    let svc = service(service_name, &shapes, &[op])?;

    let item = item_select(vec![Value::Int(1)])?;
    let extension = runtime_extension(vec![
        plan(service_name, operation_name, "vpc", 1, 1, item, identity, pagination)?
    ])?;

    let mut root = root(svc, extension)?;
    Ok(root)
}

/// Build a synthetic catalog with input/output shapes that have members at specific field IDs.
/// Supports tests that require pagination paths at specific field IDs.
pub fn pagination_catalog(input_fields: &[u32], output_fields: &[u32], pagination: Value) -> TestResult<Vec<u8>> {
    let service_name = "TestService";
    let operation_name = "ListThings";

    let mut input_members = Vec::new();
    for &field_id in input_fields {
        input_members.push(structure_member(field_id, -(i64::try_from(2)?) - 1, false)?);
    }
    let input_shape = structure_shape("Input", &input_members)?;

    let item_string = string_shape("ItemString")?;

    let mut output_members = Vec::new();
    for &field_id in output_fields {
        output_members.push(structure_member(field_id, -(i64::try_from(3)?) - 1, false)?);
    }
    let output_shape = structure_shape("Output", &output_members)?;

    let shapes = vec![input_shape, item_string, output_shape];
    let op = operation(operation_name, -(i64::try_from(1)?) - 1, -(i64::try_from(3)?) - 1)?;
    let svc = service(service_name, &shapes, &[op])?;

    let item = item_select(vec![Value::Int(1)])?;
    let identity = identity_plan(2, vec![Value::List(vec![Value::Int(1)])])?;
    let extension = runtime_extension(vec![
        plan(service_name, operation_name, "vpc", 1, 1, item, identity, pagination)?
    ])?;

    let mut root = root(svc, extension)?;
    Ok(root)
}

/// Build a synthetic catalog with an output shape that has members at specific field IDs.
/// Supports tests that require identity paths at specific field IDs.
pub fn identity_catalog(output_fields: &[u32], identity: Value) -> TestResult<Vec<u8>> {
    let service_name = "TestService";
    let operation_name = "ListThings";

    let input_shape = structure_shape("Input", &[])?;
    let item_string = string_shape("ItemString")?;

    let mut output_members = Vec::new();
    for &field_id in output_fields {
        output_members.push(structure_member(field_id, -(i64::try_from(2)?) - 1, false)?);
    }
    let output_shape = structure_shape("Output", &output_members)?;

    let shapes = vec![input_shape, item_string, output_shape];
    let op = operation(operation_name, -(i64::try_from(1)?) - 1, -(i64::try_from(3)?) - 1)?;
    let svc = service(service_name, &shapes, &[op])?;

    let item = item_select(vec![Value::Int(1)])?;
    let pagination = pagination_plan(vec![])?;
    let extension = runtime_extension(vec![
        plan(service_name, operation_name, "vpc", 1, 1, item, identity, pagination)?
    ])?;

    let mut root = root(svc, extension)?;
    Ok(root)
}