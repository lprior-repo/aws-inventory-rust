use super::pipeline::BenchResult;
use inventory_core::{
    catalog::{Catalog, MAX_CATALOG_BYTES, Target, runtime::compiled_inputs_sha256},
    record::{FieldId, Key, MapRef, Value, ValueRef, encode_container},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

// Reuse the checked-in real AWS wire model; only benchmark runtime policy is
// generated as typed binary data. No source TOML/JSON or compiler dependency.
fn find_table_names_field(catalog: &Catalog) -> BenchResult<FieldId> {
    let service = catalog
        .service("dynamodb")?
        .ok_or("DynamoDB wire model is absent")?;
    let operation = service
        .operations()?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|operation| matches!(operation.name(), Ok("ListTables")))
        .ok_or("ListTables wire model is absent")?;
    let Target::Shape(index) = operation.output()? else {
        return Err("ListTables output is not a structure".into());
    };
    let shape = service.shape(index)?.ok_or("output shape missing")?;
    Ok(shape
        .members()?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|member| matches!(member.name(), Ok("TableNames")))
        .ok_or("TableNames member missing")?
        .id)
}

fn build_bench_policy(field: FieldId) -> BenchResult<Value> {
    let schema = map([
        (1, text("AWS::DynamoDB::Table")),
        (2, text("benchmark_tables")),
        (3, Value::List(vec![])),
        (4, Value::Null),
    ])?;
    let decision = map([
        (1, text("dynamodb")),
        (2, text("ListTables")),
        (3, Value::Int(0)),
    ])?;
    let coverage = map([
        (1, Value::Null),
        (2, text("AWS::DynamoDB::Table")),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])?;
    map([
        (1, Value::Int(4)),
        (2, Value::List(vec![plan(field)?])),
        (3, Value::List(vec![schema])),
        (4, Value::List(vec![])),
        (5, Value::List(vec![coverage])),
        (6, Value::List(vec![decision])),
    ])
}

pub(super) fn bytes() -> BenchResult<Vec<u8>> {
    let base = include_bytes!("../../../catalog/aws-api.catalog");
    let catalog = Catalog::borrow(base)?;
    let field = find_table_names_field(&catalog)?;
    let mut metadata = build_bench_policy(field)?;
    let Value::Map(mut root) =
        ValueRef::Map(MapRef::from_container_bytes(base, MAX_CATALOG_BYTES)?).to_value()?
    else {
        return Err("wire catalog is not a map".into());
    };
    let extension = Key::Field(FieldId::new(60_000)?);
    root.remove(&extension);
    // The receipt names this fixture as an assembled catalog, never as an upstream source pin.
    let models = Sha256::digest(encode_container(
        &Value::Map(root.clone()),
        MAX_CATALOG_BYTES,
    )?)
    .to_vec();
    let encoded = encode_container(&metadata, MAX_CATALOG_BYTES)?;
    let reviewed =
        compiled_inputs_sha256(MapRef::from_container_bytes(&encoded, MAX_CATALOG_BYTES)?)?
            .to_vec();
    let fixture = Sha256::digest(include_bytes!("pipeline_test_catalog.rs")).to_vec();
    let receipt = map([
        (1, Value::Int(1)),
        (2, map([(1, Value::Int(1)), (2, Value::Bytes(models))])?),
        (3, Value::Int(1)),
        (4, Value::Bytes(fixture.clone())),
        (5, Value::Null),
        (6, Value::Bytes(fixture)),
        (7, Value::Bytes(reviewed)),
    ])?;
    let Value::Map(fields) = &mut metadata else {
        return Err("runtime extension is not a map".into());
    };
    fields.insert(Key::Field(FieldId::new(7)?), receipt);
    root.insert(extension, metadata);
    Ok(encode_container(&Value::Map(root), MAX_CATALOG_BYTES)?)
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}
fn map(fields: impl IntoIterator<Item = (u32, Value)>) -> BenchResult<Value> {
    Ok(Value::Map(
        fields
            .into_iter()
            .map(|(key, value)| Ok((Key::Field(FieldId::new(key)?), value)))
            .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?,
    ))
}
fn literal(value: &str) -> BenchResult<Value> {
    map([(1, Value::Int(0)), (2, text(value))])
}
fn context(kind: i64) -> BenchResult<Value> {
    map([(1, Value::Int(kind))])
}
fn dynamodb_identity() -> BenchResult<Value> {
    map([
        (1, Value::Int(1)),
        (
            2,
            Value::List(vec![
                literal("arn:")?,
                context(2)?,
                literal(":dynamodb:")?,
                context(4)?,
                literal(":")?,
                context(3)?,
                literal(":table/")?,
                map([(1, Value::Int(1)), (2, Value::List(vec![]))])?,
            ]),
        ),
    ])
}

fn dynamodb_endpoint() -> BenchResult<Value> {
    map([
        (1, text("https://dynamodb.{region}.amazonaws.com")),
        (2, text("{region}")),
        (3, text("dynamodb")),
        (4, Value::Int(1)),
        (5, Value::Int(0)),
    ])
}

fn dynamodb_metadata() -> BenchResult<Value> {
    map([
        (1, text("typed benchmark fixture")),
        (2, text("benchmark")),
        (3, text("2026-11-15")),
        (4, text("fixture")),
        (5, Value::Int(0)),
        (6, Value::Int(0)),
        (7, Value::Int(0)),
        (8, Value::Int(0)),
    ])
}

fn plan(field: FieldId) -> BenchResult<Value> {
    let identity = dynamodb_identity()?;
    map([
        (1, text("dynamodb")),
        (2, text("ListTables")),
        (3, text("AWS::DynamoDB::Table")),
        (4, Value::Int(0)),
        (
            5,
            map([
                (1, Value::Int(1)),
                (2, Value::List(vec![Value::Int(i64::from(field.value()))])),
            ])?,
        ),
        (6, identity),
        (7, Value::List(vec![])),
        (
            8,
            map([
                (
                    1,
                    map([
                        (1, Value::Int(1)),
                        (2, Value::List(vec![Value::Int(i64::from(field.value()))])),
                    ])?,
                ),
                (2, Value::List(vec![])),
            ])?,
        ),
        (9, Value::List(vec![])),
        (10, dynamodb_endpoint()?),
        (11, dynamodb_metadata()?),
        (12, map([(1, Value::Int(0))])?),
        (13, Value::Int(0)),
    ])
}
