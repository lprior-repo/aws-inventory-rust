mod model;

use inventory_core::catalog::{
    CATALOG_VERSION, CatalogKey,
    runtime::{RuntimeCatalog, compiled_inputs_sha256},
};
use inventory_core::record::{FieldId, Key, Record, Value};
use inventory_engine::inventory::SchemaRegistry;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn registry(count: u32, first_table: Option<&str>) -> TestResult<(SchemaRegistry, Vec<u8>)> {
    let mut root = base(count)?;
    let models = Sha256::digest(Record::encode(&root)?.as_bytes());
    let mut extension = extension(count, first_table)?;
    let encoded = Record::encode(&extension)?;
    let reviewed = compiled_inputs_sha256(encoded.root())?;
    let source = Sha256::digest(include_bytes!("mod.rs"));
    map_mut(&mut extension)?.insert(key(7)?, provenance(&models, &source, &reviewed)?);
    map_mut(&mut root)?.insert(key(60_000)?, extension);
    let bytes = Record::encode(&root)?.into_bytes();
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    Ok((
        SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?,
        bytes,
    ))
}

fn base(count: u32) -> TestResult<Value> {
    let fields = [
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (
            CatalogKey::ModelsCommit,
            Value::Text("synthetic-postgres-fixture".into()),
        ),
        (CatalogKey::FieldNames, Value::Bytes(b"ABCDEF".to_vec())),
        (CatalogKey::FieldNameOffsets, field_offsets()),
        (
            CatalogKey::Services,
            Value::List(vec![model::encoded_service(count)?]),
        ),
    ]
    .into_iter()
    .map(|(field, value)| Ok((Key::Field(field.field()?), value)))
    .collect::<TestResult<BTreeMap<_, _>>>()?;
    Ok(Value::Map(fields))
}

fn field_offsets() -> Value {
    Value::Bytes(
        (0_u32..6)
            .zip(1_u32..=6)
            .flat_map(|(start, end)| [start, end])
            .flat_map(u32::to_be_bytes)
            .collect(),
    )
}

fn extension(count: u32, first_table: Option<&str>) -> TestResult<Value> {
    wire([
        (1, Value::Int(2)),
        (2, values(count, model::plan)?),
        (3, values(count, |index| schema(index, first_table))?),
        (4, Value::List(Vec::new())),
        (5, values(count, coverage)?),
        (6, values(count, model::decision)?),
    ])
}

fn values(count: u32, make: impl Fn(u32) -> TestResult<Value>) -> TestResult<Value> {
    (0..count)
        .map(make)
        .collect::<TestResult<Vec<_>>>()
        .map(Value::List)
}

fn schema(index: u32, first_table: Option<&str>) -> TestResult<Value> {
    let table = match (index, first_table) {
        (0, Some(table)) => table.to_owned(),
        _ => format!("kind_{index}"),
    };
    let columns = (0_i64..6)
        .map(|scalar| {
            let field = scalar.checked_add(1).ok_or("fixture field overflow")?;
            wire([
                (1, Value::List(vec![Value::Int(field)])),
                (2, Value::Int(scalar)),
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    wire([
        (1, Value::Text(format!("AWS::Example::Kind{index}"))),
        (2, Value::Text(table)),
        (3, Value::List(columns)),
        (4, Value::Null),
    ])
}

fn coverage(index: u32) -> TestResult<Value> {
    wire([
        (1, Value::Null),
        (2, Value::Text(format!("AWS::Example::Kind{index}"))),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])
}

fn provenance(models: &[u8], source: &[u8], reviewed: &[u8]) -> TestResult<Value> {
    wire([
        (1, Value::Int(1)),
        (
            2,
            wire([(1, Value::Int(1)), (2, Value::Bytes(models.to_vec()))])?,
        ),
        (3, Value::Int(1)),
        (4, Value::Bytes(source.to_vec())),
        (5, Value::Null),
        (6, Value::Bytes(source.to_vec())),
        (7, Value::Bytes(reviewed.to_vec())),
    ])
}

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn map_mut(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("fixture map missing".into()),
    }
}

fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    fields
        .into_iter()
        .map(|(id, value)| Ok((key(id)?, value)))
        .collect::<TestResult<BTreeMap<_, _>>>()
        .map(Value::Map)
}
