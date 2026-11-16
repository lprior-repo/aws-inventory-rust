use super::fixtures::TestResult;
use inventory_core::{
    catalog::{CATALOG_VERSION, CatalogKey},
    record::{FieldId, Key, Record, Value},
};
use std::collections::BTreeMap;

pub(super) fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(fields.into_iter().map(|(id, value)| {
        Ok((Key::Field(FieldId::new(id)?), value))
    }).collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?))
}

fn fields(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    wire(entries.into_iter().map(|(key, value)| Ok((key.field()?.value(), value)))
        .collect::<TestResult<Vec<_>>>()?)
}

pub(super) fn catalog() -> TestResult<Vec<u8>> {
    let plan = wire([
        (1, Value::Text("example".into())), (2, Value::Text("ListThings".into())),
        (3, Value::Text(super::fixtures::KIND.into())), (4, Value::Int(0)),
        (5, wire([(1, Value::Int(0))])?),
        (6, wire([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?),
        (7, Value::List(vec![])),
        (8, wire([(1, wire([(1, Value::Int(0))])?), (2, Value::List(vec![]))])?),
        (9, Value::List(vec![])),
        (10, wire([
            (1, Value::Text("https://example.invalid".into())),
            (2, Value::Text("us-east-1".into())), (3, Value::Text("example".into())),
            (4, Value::Int(0)), (5, Value::Int(0)),
        ])?),
        (11, wire([
            (1, Value::Text("real UI worker fixture".into())),
            (2, Value::Text("test".into())), (3, Value::Text("2026-11-15".into())),
        ])?),
        (12, wire([(1, Value::Int(0))])?), (13, Value::Int(5)),
    ])?;
    let schema = wire([
        (1, Value::Text(super::fixtures::KIND.into())),
        (2, Value::Text("thing_records".into())),
        (3, Value::List(vec![wire([(1, Value::List(vec![Value::Int(1)])), (2, Value::Int(3))])?])),
        (4, Value::Null),
    ])?;
    let coverage = wire([
        (1, Value::Null), (2, Value::Text(super::fixtures::KIND.into())),
        (3, Value::Bool(true)), (4, Value::Null), (5, Value::Bool(true)),
    ])?;
    let decision = wire([
        (1, Value::Text("example".into())), (2, Value::Text("ListThings".into())),
        (3, Value::Int(0)),
    ])?;
    let runtime = wire([
        (1, Value::Int(2)), (2, Value::List(vec![plan])),
        (3, Value::List(vec![schema])), (4, Value::List(vec![])),
        (5, Value::List(vec![coverage])), (6, Value::List(vec![decision])),
    ])?;
    let operation = fields([
        (CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)), (CatalogKey::Input, Value::Int(-1)),
        (CatalogKey::Output, Value::Int(-14)),
    ])?;
    let service = fields([
        (CatalogKey::Name, Value::Text("example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#example".into())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".into())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(vec![])),
        (CatalogKey::ShapeOffsets, Value::Bytes(vec![])),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let offsets = (0_u32..8).flat_map(|index| [index * 2, (index + 1) * 2])
        .flat_map(u32::to_be_bytes).collect();
    let root = fields([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text("real-worker-contract".into())),
        (CatalogKey::FieldNames, Value::Bytes(b"a1a2a3a4a5a6a7a8".to_vec())),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())])),
    ])?;
    let Value::Map(mut entries) = root else { return Err("catalog map required".into()); };
    entries.insert(Key::Field(FieldId::new(60_000)?), runtime);
    Ok(Record::encode(&Value::Map(entries))?.into_bytes())
}
