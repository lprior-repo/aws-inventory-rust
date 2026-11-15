use inventory_core::{catalog::{Catalog, Target}, record::{FieldId, Key, MapRef, Record, Value, ValueRef}};
use std::collections::BTreeMap;
use super::pipeline::BenchResult;

// Reuse the checked-in real AWS wire model; only benchmark runtime policy is
// generated as typed binary data. No source TOML/JSON or compiler dependency.
pub(super) fn bytes() -> BenchResult<Vec<u8>> {
    let base = include_bytes!("../../../catalog/aws-api.catalog");
    let catalog = Catalog::borrow(base)?;
    let service = catalog.service("dynamodb")?.ok_or("DynamoDB wire model is absent")?;
    let operation = service.operations()?.collect::<Result<Vec<_>, _>>()?.into_iter()
        .find(|operation| matches!(operation.name(), Ok("ListTables"))).ok_or("ListTables wire model is absent")?;
    let Target::Shape(index) = operation.output()? else { return Err("ListTables output is not a structure".into()); };
    let shape = service.shape(index)?.ok_or("output shape missing")?;
    let field = shape.members()?.collect::<Result<Vec<_>, _>>()?.into_iter()
        .find(|member| matches!(member.name(), Ok("TableNames"))).ok_or("TableNames member missing")?.id;
    let schema = map([(1, text("AWS::DynamoDB::Table")), (2, text("benchmark_tables")),
        (3, Value::List(vec![])), (4, Value::Null)])?;
    let decision = map([(1, text("dynamodb")), (2, text("ListTables")), (3, Value::Int(0))])?;
    let metadata = map([(1, Value::Int(2)), (2, Value::List(vec![plan(field)?])), (3, Value::List(vec![schema])),
        (4, Value::List(vec![])), (5, Value::List(vec![])), (6, Value::List(vec![decision]))])?;
    let Value::Map(mut root) = ValueRef::Map(MapRef::from_bytes_bounded(base)?).to_value()? else { return Err("wire catalog is not a map".into()); };
    root.insert(Key::Field(FieldId::new(60_000)?), metadata);
    Ok(Record::encode(&Value::Map(root))?.into_bytes())
}

fn text(value: &str) -> Value { Value::Text(value.to_owned()) }
fn map(fields: impl IntoIterator<Item = (u32, Value)>) -> BenchResult<Value> {
    Ok(Value::Map(fields.into_iter().map(|(key, value)| Ok((Key::Field(FieldId::new(key)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?))
}
fn literal(value: &str) -> BenchResult<Value> { map([(1, Value::Int(0)), (2, text(value))]) }
fn context(kind: i64) -> BenchResult<Value> { map([(1, Value::Int(kind))]) }
fn plan(field: FieldId) -> BenchResult<Value> {
    let identity = map([(1, Value::Int(1)), (2, Value::List(vec![literal("arn:")?, context(2)?, literal(":dynamodb:")?,
        context(4)?, literal(":")?, context(3)?, literal(":table/")?, map([(1, Value::Int(1)), (2, Value::List(vec![]))])?]))])?;
    map([
        (1, text("dynamodb")), (2, text("ListTables")), (3, text("AWS::DynamoDB::Table")), (4, Value::Int(0)),
        (5, map([(1, Value::Int(1)), (2, Value::List(vec![Value::Int(i64::from(field.value()))]))])?),
        (6, identity), (7, Value::List(vec![])),
        (8, map([(1, map([(1, Value::Int(1)), (2, Value::List(vec![Value::Int(i64::from(field.value()))]))])?), (2, Value::List(vec![]))])?), (9, Value::List(vec![])),
        (10, map([(1, text("https://dynamodb.{region}.amazonaws.com")), (2, text("{region}")), (3, text("dynamodb")), (4, Value::Int(1)), (5, Value::Int(0))])?),
        (11, map([(1, text("typed benchmark fixture")), (2, text("benchmark")), (3, text("2026-11-15"))])?),
        (12, map([(1, Value::Int(0))])?), (13, Value::Int(0)),
    ])
}
