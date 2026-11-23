use super::{TestResult, wire};
use inventory_core::catalog::CatalogKey;
use inventory_core::record::{Key, ListRef, Record, Value, encode_list};
use std::collections::BTreeMap;

pub(super) fn service(count: u32) -> TestResult<Value> {
    let shapes = vec![shape("Input", Vec::new())?, output()?];
    let bytes = encode_list(&shapes)?;
    let offsets = ListRef::from_bytes(&bytes)?
        .encoded_item_offsets()?
        .iter()
        .flat_map(|offset| offset.to_be_bytes())
        .collect();
    let operations = (0..count).map(operation).collect::<TestResult<Vec<_>>>()?;
    catalog_map([
        (CatalogKey::Name, Value::Text("example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#example".into())),
        (
            CatalogKey::Protocols,
            Value::List(vec![Value::Text("aws.protocols#restJson1".into())]),
        ),
        (CatalogKey::Operations, Value::List(operations)),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, Value::Bytes(offsets)),
        (CatalogKey::Resources, Value::List(Vec::new())),
    ])
}

fn shape(name: &str, members: Vec<Value>) -> TestResult<Value> {
    catalog_map([
        (CatalogKey::Name, Value::Text(name.into())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(members)),
    ])
}

fn output() -> TestResult<Value> {
    let members = (1_i64..=6)
        .zip([-2, -7, -9, -3, -1, -12])
        .map(|(id, target)| {
            catalog_map([
                (CatalogKey::MemberId, Value::Int(id)),
                (CatalogKey::MemberTarget, Value::Int(target)),
                (CatalogKey::MemberRequired, Value::Bool(false)),
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    shape("Output", members)
}

fn operation(index: u32) -> TestResult<Value> {
    catalog_map([
        (CatalogKey::Name, Value::Text(format!("ListThings{index}"))),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(0)),
        (CatalogKey::Output, Value::Int(1)),
    ])
}

pub(super) fn plan(index: u32) -> TestResult<Value> {
    wire([
        (1, Value::Text("example".into())),
        (2, Value::Text(format!("ListThings{index}"))),
        (3, Value::Text(format!("AWS::Example::Kind{index}"))),
        (4, Value::Int(0)),
        (5, wire([(1, Value::Int(0))])?),
        (
            6,
            wire([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(4)]))])?,
        ),
        (7, Value::List(Vec::new())),
        (8, pagination()?),
        (9, Value::List(Vec::new())),
        (10, endpoint()?),
        (11, review()?),
        (12, wire([(1, Value::Int(0))])?),
        (13, Value::Int(5)),
    ])
}

fn pagination() -> TestResult<Value> {
    wire([
        (1, wire([(1, Value::Int(0))])?),
        (2, Value::List(Vec::new())),
    ])
}

pub(super) fn decision(index: u32) -> TestResult<Value> {
    wire([
        (1, Value::Text("example".into())),
        (2, Value::Text(format!("ListThings{index}"))),
        (3, Value::Int(0)),
    ])
}

fn endpoint() -> TestResult<Value> {
    wire([
        (1, Value::Text("https://example.invalid".into())),
        (2, Value::Text("us-east-1".into())),
        (3, Value::Text("example".into())),
        (4, Value::Int(0)),
        (5, Value::Int(0)),
    ])
}

fn review() -> TestResult<Value> {
    wire([
        (1, Value::Text("synthetic PostgreSQL fixture".into())),
        (2, Value::Text("reviewer".into())),
        (3, Value::Text("2026-01-01".into())),
        (4, Value::Text("fixture".into())),
        (5, Value::Int(0)),
        (6, Value::Int(0)),
        (7, Value::Int(0)),
        (8, Value::Int(0)),
    ])
}

fn catalog_map(fields: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    fields
        .into_iter()
        .map(|(field, value)| Ok((Key::Field(field.field()?), value)))
        .collect::<TestResult<BTreeMap<_, _>>>()
        .map(Value::Map)
}

pub(super) fn encoded_service(count: u32) -> TestResult<Value> {
    Ok(Value::Bytes(Record::encode(&service(count)?)?.into_bytes()))
}
