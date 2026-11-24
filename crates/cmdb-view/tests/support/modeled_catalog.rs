use inventory_core::catalog::{CATALOG_VERSION, CatalogKey, runtime::compiled_inputs_sha256};
use inventory_core::record::{FieldId, Key, ListRef, Record, Value, encode_list};
use sha2::Digest;
use std::{collections::BTreeMap, error::Error};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub struct Model<'a> {
    pub kind: &'a str,
    pub region: &'a str,
    pub table: &'a str,
    pub commit: &'a str,
}

fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        fields
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<TestResult<BTreeMap<_, _>>>()?,
    ))
}

fn fields(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    wire(
        entries
            .into_iter()
            .map(|(key, value)| Ok((key.field()?.value(), value)))
            .collect::<TestResult<Vec<_>>>()?,
    )
}

fn text(value: &str) -> Value {
    Value::Text(value.into())
}

pub fn modeled_catalog(model: &Model) -> TestResult<Vec<u8>> {
    let member = |id, target| {
        fields([
            (CatalogKey::MemberId, Value::Int(id)),
            (CatalogKey::MemberTarget, Value::Int(target)),
            (CatalogKey::MemberRequired, Value::Bool(false)),
        ])
    };
    let input = fields([
        (CatalogKey::Name, text("Input")),
        (CatalogKey::Kind, Value::Int(14)),
        (
            CatalogKey::Members,
            Value::List(vec![member(2, 2)?, member(3, -3)?]),
        ),
    ])?;
    let output = fields([
        (CatalogKey::Name, text("Thing")),
        (CatalogKey::Kind, Value::Int(14)),
        (CatalogKey::Members, Value::List(vec![member(1, -3)?])),
    ])?;
    let names = fields([
        (CatalogKey::Name, text("Names")),
        (CatalogKey::Kind, Value::Int(16)),
        (CatalogKey::Element, Value::Int(-3)),
    ])?;
    let shapes = vec![input, output, names];
    let shape_bytes = encode_list(&shapes)?;
    let shape_offsets = ListRef::from_bytes(&shape_bytes)?
        .encoded_item_offsets()?
        .iter()
        .flat_map(|offset| offset.to_be_bytes())
        .collect();
    let operation = fields([
        (CatalogKey::Name, text("ListThings")),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(0)),
        (CatalogKey::Output, Value::Int(1)),
    ])?;
    let service = fields([
        (CatalogKey::Name, text("example")),
        (CatalogKey::ApiVersion, text("1")),
        (CatalogKey::ServiceShape, text("n#example")),
        (
            CatalogKey::Protocols,
            Value::List(vec![text("aws.protocols#restJson1")]),
        ),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, Value::Bytes(shape_offsets)),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let offsets = [0_u32, 4, 4, 18, 18, 33, 33, 38, 38, 46]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    let mut root = fields([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, text(model.commit)),
        (
            CatalogKey::FieldNames,
            Value::Bytes(b"NameReferenceValueDependencyValueAliasRevision".to_vec()),
        ),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())]),
        ),
    ])?;
    let Value::Map(entries) = &mut root else {
        return Err("catalog map required".into());
    };
    entries.insert(Key::Field(FieldId::new(60_000)?), runtime(model)?);
    seal(&mut root)?;
    Ok(Record::encode(&root)?.into_bytes())
}

fn runtime(model: &Model) -> TestResult<Value> {
    let schema = wire([
        (1, text(model.kind)),
        (2, text(model.table)),
        (
            3,
            Value::List(vec![wire([
                (1, Value::List(vec![Value::Int(1)])),
                (2, Value::Int(3)),
            ])?]),
        ),
        (4, Value::Null),
    ])?;
    let coverage = wire([
        (1, Value::Null),
        (2, text(model.kind)),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])?;
    let relations = [(2, "references"), (3, "depends-on")]
        .into_iter()
        .map(|(source, relation)| {
            wire([
                (1, text(model.kind)),
                (2, Value::List(vec![Value::Int(source)])),
                (3, text(model.kind)),
                (4, Value::List(vec![Value::Int(1)])),
                (5, text(relation)),
                (6, Value::Int(0)),
                (7, Value::Int(0)),
                (10, Value::Int(1)),
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    let decision = wire([
        (1, text("example")),
        (2, text("ListThings")),
        (3, Value::Int(0)),
    ])?;
    wire([
        (1, Value::Int(2)),
        (2, Value::List(vec![plan(model)?])),
        (3, Value::List(vec![schema])),
        (4, Value::List(relations)),
        (5, Value::List(vec![coverage])),
        (6, Value::List(vec![decision])),
    ])
}

fn plan(model: &Model) -> TestResult<Value> {
    wire([
        (1, text("example")),
        (2, text("ListThings")),
        (3, text(model.kind)),
        (4, Value::Int(0)),
        (5, wire([(1, Value::Int(0))])?),
        (
            6,
            wire([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?,
        ),
        (7, Value::List(vec![])),
        (
            8,
            wire([(1, wire([(1, Value::Int(0))])?), (2, Value::List(vec![]))])?,
        ),
        (9, Value::List(vec![])),
        (
            10,
            wire([
                (1, text("https://example.invalid")),
                (2, text(model.region)),
                (3, text("example")),
                (4, Value::Int(0)),
                (5, Value::Int(0)),
            ])?,
        ),
        (
            11,
            wire([
                (1, text("modeled fixture")),
                (2, text("test")),
                (3, text("2026-11-15")),
                (4, text("fixture")),
                (5, Value::Int(0)),
                (6, Value::Int(0)),
                (7, Value::Int(0)),
                (8, Value::Int(0)),
            ])?,
        ),
        (12, wire([(1, Value::Int(0))])?),
        (13, Value::Int(5)),
    ])
}

fn seal(root: &mut Value) -> TestResult {
    let Value::Map(entries) = root else {
        return Err("catalog root is not a map".into());
    };
    let mut base = entries.clone();
    base.remove(&Key::Field(FieldId::new(60_000)?));
    let model_hash = sha2::Sha256::digest(Record::encode(&Value::Map(base))?.as_bytes());
    let extension = entries
        .get_mut(&Key::Field(FieldId::new(60_000)?))
        .ok_or("runtime extension missing")?;
    let reviewed = compiled_inputs_sha256(Record::encode(extension)?.root())?;
    let source_hash = sha2::Sha256::digest(include_bytes!("modeled_catalog.rs"));
    let Value::Map(extension) = extension else {
        return Err("runtime extension is not a map".into());
    };
    extension.insert(
        Key::Field(FieldId::new(7)?),
        wire([
            (1, Value::Int(1)),
            (
                2,
                wire([(1, Value::Int(1)), (2, Value::Bytes(model_hash.to_vec()))])?,
            ),
            (3, Value::Int(1)),
            (4, Value::Bytes(source_hash.to_vec())),
            (5, Value::Null),
            (6, Value::Bytes(source_hash.to_vec())),
            (7, Value::Bytes(reviewed.to_vec())),
        ])?,
    );
    Ok(())
}
