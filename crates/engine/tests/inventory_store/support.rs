use inventory_core::catalog::{CATALOG_VERSION, CatalogKey};
use inventory_core::inventory::{
    AccountId, Column, EdgeSource, Kind, KindSchema, Observation, OperationId, ReferenceTarget,
    Region, Relation, ResourceId, ResourceKey, ScalarType, ScopeKey, ServiceId, SqlName,
    UnresolvedReference,
};
use inventory_core::record::{Exclusions, FieldId, Key, Record, Value};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use sha2::Digest;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) async fn fixture() -> TestResult<(tempfile::TempDir, BinaryStore, Vec<u8>)> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let catalog = catalog_bytes()?;
    let column = Column::new(vec![FieldId::new(1)?], ScalarType::Text)?;
    let kind = KindSchema::new(
        Kind::parse("AWS::Example::Thing")?,
        SqlName::parse("thing_table")?,
        vec![column],
    )?;
    let registry = SchemaRegistry::new(&catalog, vec![Arc::new(kind)])?;
    let store = BinaryStore::create(&database, registry).await?;
    Ok((directory, store, catalog))
}

pub(super) fn scope(region: &str) -> TestResult<ScopeKey> {
    Ok(ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?)
}

pub(super) fn work(sweep: &str, region: &str) -> TestResult<WorkSpec> {
    WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope(region)?,
        None,
        vec![1, 2],
    )
    .map_err(Into::into)
}

pub(super) fn child_work(sweep: &str, parent: &ResourceKey) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(
        parent.account.clone(),
        parent.region.clone(),
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        Some(parent.clone()),
    )?;
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        vec![1],
    )?)
}

pub(super) fn resource(id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::Example::Thing")?,
        id: ResourceId::parse(id)?,
    })
}

pub(super) fn observation(id: &str, value: &str) -> TestResult<Observation> {
    let field = FieldId::new(1)?;
    let fields = BTreeMap::from([(Key::Field(field), Value::Text(value.to_owned()))]);
    let record = Record::encode(&Value::Map(fields))?;
    Ok(Observation::new(
        resource(id)?,
        record,
        &Exclusions::none(),
        None,
    )?)
}

pub(super) fn unresolved(from: &str, target: &str) -> TestResult<UnresolvedReference> {
    Ok(UnresolvedReference {
        from: resource(from)?,
        relation: Relation::parse("references")?,
        source: EdgeSource::InferredExact,
        target: ReferenceTarget::new(
            AccountId::parse("123456789012")?,
            Region::parse("us-east-1")?,
            Kind::parse("AWS::Example::Thing")?,
            vec![FieldId::new(1)?],
            ResourceId::parse(target)?,
        )?,
    })
}

pub(super) fn page(
    lease: inventory_engine::inventory::WorkLease,
    observations: Vec<Observation>,
    done: bool,
    next_token: Option<Vec<u8>>,
    at: i64,
) -> PageCommit {
    let continuation_digest = next_token
        .as_ref()
        .map(|token| <[u8; 32]>::from(sha2::Sha256::digest(token.as_slice())));
    PageCommit {
        lease,
        observations,
        edges: Vec::new(),
        unresolved: Vec::new(),
        tags: Vec::new(),
        next_token,
        continuation_digest,
        children: Vec::new(),
        done,
        observed_at_ms: at,
    }
}

pub(super) fn catalog_bytes() -> TestResult<Vec<u8>> {
    let operation = catalog_map([
        (CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(-1)),
        (CatalogKey::Output, Value::Int(-14)),
    ])?;
    let service = catalog_map([
        (CatalogKey::Name, Value::Text("example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#example".into())),
        (
            CatalogKey::Protocols,
            Value::List(vec![Value::Text("aws.protocols#restJson1".into())]),
        ),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(vec![])),
        (CatalogKey::ShapeOffsets, Value::Bytes(Vec::new())),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let (names, offsets) = field_registry_values();
    let service_bytes = Record::encode(&service)?.into_bytes();
    let base = catalog_map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (
            CatalogKey::ModelsCommit,
            Value::Text("reviewed-test-catalog".into()),
        ),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(service_bytes)]),
        ),
    ])?;
    let mut root = match base {
        Value::Map(fields) => fields,
        _ => return Err("catalog root is not a map".into()),
    };
    root.insert(Key::Field(FieldId::new(60_000)?), runtime_metadata()?);
    Ok(Record::encode(&Value::Map(root))?.into_bytes())
}

fn runtime_metadata() -> TestResult<Value> {
    let column = wire_map([(1, Value::List(vec![Value::Int(1)])), (2, Value::Int(3))])?;
    let schema = wire_map([
        (1, Value::Text("AWS::Example::Thing".into())),
        (2, Value::Text("thing_table".into())),
        (3, Value::List(vec![column])),
        (4, Value::Null),
    ])?;
    let coverage = wire_map([
        (1, Value::Null),
        (2, Value::Text("AWS::Example::Thing".into())),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])?;
    let plan = callable_plan()?;
    let decision = wire_map([
        (1, Value::Text("example".into())),
        (2, Value::Text("ListThings".into())),
        (3, Value::Int(0)),
    ])?;
    wire_map([
        (1, Value::Int(1)),
        (2, Value::List(vec![plan])),
        (3, Value::List(vec![schema])),
        (4, Value::List(vec![])),
        (5, Value::List(vec![coverage])),
        (6, Value::List(vec![decision])),
    ])
}

fn callable_plan() -> TestResult<Value> {
    wire_map([
        (1, Value::Text("example".into())),
        (2, Value::Text("ListThings".into())),
        (3, Value::Text("AWS::Example::Thing".into())),
        (4, Value::Int(0)),
        (5, wire_map([(1, Value::Int(0))])?),
        (
            6,
            wire_map([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?,
        ),
        (7, Value::List(vec![])),
        (
            8,
            wire_map([
                (1, wire_map([(1, Value::Int(0))])?),
                (2, Value::List(vec![])),
            ])?,
        ),
        (9, Value::List(vec![])),
        (
            10,
            wire_map([
                (1, Value::Text("https://example.invalid".into())),
                (2, Value::Text("us-east-1".into())),
                (3, Value::Text("example".into())),
                (4, Value::Int(0)),
                (5, Value::Int(0)),
            ])?,
        ),
        (
            11,
            wire_map([
                (1, Value::Text("test catalog".into())),
                (2, Value::Text("reviewer".into())),
                (3, Value::Text("2026-01-01".into())),
            ])?,
        ),
        (12, wire_map([(1, Value::Int(0))])?),
        (13, Value::Int(5)),
    ])
}

fn wire_map(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    let pairs = fields
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Value::Map(pairs))
}

fn catalog_map(fields: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    let pairs = fields
        .into_iter()
        .map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Value::Map(pairs))
}

fn field_registry_values() -> (Value, Value) {
    let offsets = (0_u32..8)
        .flat_map(|index| [index * 2, (index + 1) * 2])
        .flat_map(u32::to_be_bytes)
        .collect();
    (
        Value::Bytes(b"a1a2a3a4a5a6a7a8".to_vec()),
        Value::Bytes(offsets),
    )
}
