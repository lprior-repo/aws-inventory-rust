use inventory_core::{
    catalog::{
        CATALOG_VERSION, CatalogKey, MAX_CATALOG_BYTES,
        runtime::{RuntimeCatalog, compiled_inputs_sha256},
    },
    record::{FieldId, Key, ListRef, MapRef, Record, Value, encode_container, encode_list},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
type Fields = BTreeMap<Key, Value>;

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

pub fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((key(id)?, value)))
            .collect::<TestResult<Fields>>()?,
    ))
}

pub fn catalog_map(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(id.field()?), value)))
            .collect::<TestResult<Fields>>()?,
    ))
}

fn fields(value: &mut Value) -> TestResult<&mut Fields> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("synthetic fixture expected a map".into()),
    }
}

pub fn path(ids: impl IntoIterator<Item = u32>) -> Value {
    Value::List(
        ids.into_iter()
            .map(|id| Value::Int(i64::from(id)))
            .collect(),
    )
}

pub fn structure(name: &str, members: Vec<Value>) -> TestResult<Value> {
    catalog_map([
        (CatalogKey::Name, Value::Text(name.to_owned())),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(members)),
    ])
}

pub fn member(id: u32, target: i64) -> TestResult<Value> {
    catalog_map([
        (CatalogKey::MemberId, Value::Int(i64::from(id))),
        (CatalogKey::MemberTarget, Value::Int(target)),
        (CatalogKey::MemberRequired, Value::Bool(false)),
    ])
}

fn service(input: u32, output: u32, shapes: Vec<Value>) -> TestResult<Value> {
    let encoded = encode_list(&shapes)?;
    let offsets = ListRef::from_bytes(&encoded)?.encoded_item_offsets()?;
    let operation = catalog_map([
        (CatalogKey::Name, Value::Text("ListThings".to_owned())),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(i64::from(input))),
        (CatalogKey::Output, Value::Int(i64::from(output))),
    ])?;
    catalog_map([
        (CatalogKey::Name, Value::Text("TestService".to_owned())),
        (CatalogKey::ApiVersion, Value::Text("1".to_owned())),
        (
            CatalogKey::ServiceShape,
            Value::Text("fixture#TestService".to_owned()),
        ),
        (
            CatalogKey::Protocols,
            Value::List(vec![Value::Text("aws.protocols#restJson1".to_owned())]),
        ),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (
            CatalogKey::ShapeOffsets,
            Value::Bytes(
                offsets
                    .iter()
                    .flat_map(|offset| offset.to_be_bytes())
                    .collect(),
            ),
        ),
        (CatalogKey::Resources, Value::List(vec![])),
    ])
}

fn root(service: Value) -> TestResult<Value> {
    let mut names = Vec::new();
    let mut offsets = Vec::new();
    for id in 1..=32 {
        offsets.extend_from_slice(&u32::try_from(names.len())?.to_be_bytes());
        names.extend_from_slice(format!("a{id}").as_bytes());
        offsets.extend_from_slice(&u32::try_from(names.len())?.to_be_bytes());
    }
    catalog_map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (
            CatalogKey::ModelsCommit,
            Value::Text("synthetic-fixture".to_owned()),
        ),
        (CatalogKey::FieldNames, Value::Bytes(names)),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())]),
        ),
    ])
}

pub fn item_root() -> TestResult<Value> {
    map([(1, Value::Int(0))])
}

fn plan(item: Value, identity: Value, mut pagination: Value) -> TestResult<Value> {
    let tokens = fields(&mut pagination)?
        .remove(&key(1)?)
        .ok_or("missing fixture tokens")?;
    let pagination = map([(1, item.clone()), (2, tokens)])?;
    map([
        (1, Value::Text("TestService".to_owned())),
        (2, Value::Text("ListThings".to_owned())),
        (3, Value::Text("AWS::EC2::VPC".to_owned())),
        (4, Value::Int(0)),
        (5, item),
        (6, identity),
        (7, Value::List(vec![])),
        (8, pagination),
        (9, Value::List(vec![])),
        (
            10,
            map([
                (
                    1,
                    Value::Text("https://fixture.invalid/{region}".to_owned()),
                ),
                (2, Value::Text("{region}".to_owned())),
                (3, Value::Text("fixture".to_owned())),
                (4, Value::Int(1)),
                (5, Value::Int(0)),
            ])?,
        ),
        (
            11,
            map([
                (1, Value::Text("synthetic fixture".to_owned())),
                (2, Value::Text("fixture".to_owned())),
                (3, Value::Text("fixture".to_owned())),
                (4, Value::Text("fixture".to_owned())),
                (5, Value::Int(0)),
                (6, Value::Int(0)),
                (7, Value::Int(0)),
                (8, Value::Int(0)),
            ])?,
        ),
        (12, map([(1, Value::Int(0))])?),
        (13, Value::Int(5)),
    ])
}

pub fn column(ids: impl IntoIterator<Item = u32>) -> TestResult<Value> {
    map([(1, path(ids)), (2, Value::Int(3))])
}

fn seal(mut root: Value, mut extension: Value) -> TestResult<Vec<u8>> {
    let base = RuntimeCatalog::from_bytes(include_bytes!("../../../../catalog/aws-api.catalog"))?;
    let source = base.source_provenance();
    let compiled = Sha256::digest(encode_container(&root, MAX_CATALOG_BYTES)?);
    let encoded = Record::encode(&extension)?;
    let reviewed = compiled_inputs_sha256(MapRef::from_bytes_bounded(encoded.as_bytes())?)?;
    fields(&mut extension)?.insert(
        key(7)?,
        map([
            (1, Value::Int(1)),
            (
                2,
                map([(1, Value::Int(1)), (2, Value::Bytes(compiled.to_vec()))])?,
            ),
            (3, Value::Int(i64::from(source.kind_map_version))),
            (4, Value::Bytes(source.kind_map_sha256.to_vec())),
            (5, Value::Null),
            (6, Value::Bytes(source.embedded_sources_sha256.to_vec())),
            (7, Value::Bytes(reviewed.to_vec())),
        ])?,
    );
    fields(&mut root)?.insert(key(60_000)?, extension);
    Ok(encode_container(&root, MAX_CATALOG_BYTES)?)
}

pub fn catalog(
    input: u32,
    output: u32,
    shapes: Vec<Value>,
    item: Value,
    identity: Value,
    pagination: Value,
    columns: Vec<Value>,
) -> TestResult<Vec<u8>> {
    let extension = map([
        (1, Value::Int(2)),
        (2, Value::List(vec![plan(item, identity, pagination)?])),
        (
            3,
            Value::List(vec![map([
                (1, Value::Text("AWS::EC2::VPC".to_owned())),
                (2, Value::Text("fixture_vpc".to_owned())),
                (3, Value::List(columns)),
            ])?]),
        ),
        (4, Value::List(vec![])),
        (
            5,
            Value::List(vec![map([
                (1, Value::Null),
                (2, Value::Text("AWS::EC2::VPC".to_owned())),
                (3, Value::Bool(true)),
                (4, Value::Null),
                (5, Value::Bool(true)),
            ])?]),
        ),
        (
            6,
            Value::List(vec![map([
                (1, Value::Text("TestService".to_owned())),
                (2, Value::Text("ListThings".to_owned())),
                (3, Value::Int(0)),
                (4, Value::Null),
            ])?]),
        ),
    ])?;
    seal(root(service(input, output, shapes)?)?, extension)
}
