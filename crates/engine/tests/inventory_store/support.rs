use inventory_core::catalog::{
    CATALOG_VERSION, CatalogKey,
    runtime::{RuntimeCatalog, compiled_inputs_sha256},
};
use inventory_core::inventory::{
    AccountId, Column, EdgeSource, Kind, KindSchema, Observation, OperationId, ReferenceTarget,
    Region, Relation, ResourceId, ResourceKey, ScalarType, ScopeKey, ServiceId, SqlName,
    UnresolvedReference,
};
use inventory_core::record::{Exclusions, FieldId, Key, ListRef, Record, Value, encode_list};
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

pub(super) async fn conformance_fixture() -> TestResult<(tempfile::TempDir, BinaryStore, Vec<u8>)> {
    let (directory, store, catalog) = fixture().await?;
    for id in ["work-1", "live-lease", "takeover", "dummy"] {
        let sweep = SweepId::new(id.to_owned())?;
        store.create_sweep_manifest(&sweep, "111111111111").await?;
    }
    Ok((directory, store, catalog))
}

pub(super) async fn dual_fixture() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let catalog = modeled_catalog(None, true)?;
    let runtime = RuntimeCatalog::from_bytes(&catalog)?;
    let registry = SchemaRegistry::new(&catalog, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
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
    let bindings = Record::encode(&Value::Map(BTreeMap::new()))?.into_bytes();
    WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope(region)?,
        None,
        bindings,
    )
    .map_err(Into::into)
}

pub(super) fn relationship_work(
    sweep: &str,
    region: &str,
    reference: Option<&str>,
    dependency: Option<&str>,
) -> TestResult<WorkSpec> {
    let fields = [(2, reference), (3, dependency)]
        .into_iter()
        .filter_map(|(id, value)| value.map(|value| (id, Value::Text(value.to_owned()))));
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope(region)?,
        None,
        Record::encode(&wire_map(fields)?)?.into_bytes(),
    )?)
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

pub(super) fn dual_observation(
    id: &str,
    name: &str,
    alias: &str,
    revision: &str,
) -> TestResult<Observation> {
    Ok(Observation::new(
        resource(id)?,
        Record::encode(&wire_map([
            (1, Value::Text(name.to_owned())),
            (4, Value::Text(alias.to_owned())),
            (5, Value::Text(revision.to_owned())),
        ])?)?,
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
    modeled_catalog(None, false)
}

pub(super) fn branch_catalog_bytes(base_discovery: bool) -> TestResult<Vec<u8>> {
    let fixed = |value: &str| {
        wire_map([
            (1, Value::List(vec![Value::Int(2)])),
            (2, Value::Int(2)),
            (3, Value::Text(value.to_owned())),
        ])
    };
    let branch = |name: &str, emission, value: &str| {
        wire_map([
            (1, Value::Text(name.to_owned())),
            (2, Value::List(vec![fixed(value)?])),
            (3, Value::Int(0)),
            (5, Value::Int(emission)),
        ])
    };
    let Value::Map(mut plan) = callable_plan(false)? else {
        return Err("synthetic callable plan is not a map".into());
    };
    plan.insert(
        Key::Field(FieldId::new(7)?),
        Value::List(vec![fixed("base-name")?]),
    );
    plan.insert(
        Key::Field(FieldId::new(16)?),
        Value::List(vec![
            branch("DISCOVER", 1, "discovery-name")?,
            branch("EMIT", 0, "emitting-name")?,
        ]),
    );
    plan.insert(
        Key::Field(FieldId::new(19)?),
        Value::Int(i64::from(base_discovery)),
    );
    if base_discovery {
        plan.insert(
            Key::Field(FieldId::new(6)?),
            wire_map([(1, Value::Int(2)), (2, Value::List(Vec::new()))])?,
        );
    }
    catalog_with_plan(Value::Map(plan))
}

pub(super) fn declared_branch_catalog_bytes(names: &[&str]) -> TestResult<Vec<u8>> {
    let branches = names
        .iter()
        .map(|name| {
            wire_map([
                (1, Value::Text((*name).to_owned())),
                (2, Value::List(Vec::new())),
                (3, Value::Int(0)),
                (5, Value::Int(0)),
            ])
        })
        .collect::<TestResult<Vec<_>>>()?;
    let Value::Map(mut plan) = callable_plan(false)? else {
        return Err("synthetic callable plan is not a map".into());
    };
    plan.insert(Key::Field(FieldId::new(16)?), Value::List(branches));
    plan.insert(
        Key::Field(FieldId::new(20)?),
        wire_map([
            (1, Value::List(vec![Value::List(vec![Value::Int(1)])])),
            (2, Value::List(Vec::new())),
            (3, Value::List(Vec::new())),
        ])?,
    );
    catalog_with_plan(Value::Map(plan))
}

pub(super) async fn declared_branch_fixture(
    names: &[&str],
) -> TestResult<(tempfile::TempDir, BinaryStore, Vec<u8>)> {
    let directory = tempfile::tempdir()?;
    let bytes = declared_branch_catalog_bytes(names)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store, bytes))
}

fn catalog_with_plan(plan: Value) -> TestResult<Vec<u8>> {
    let record = Record::from_bytes(catalog_bytes()?)?;
    let mut root = inventory_core::record::ValueRef::Map(record.root()).to_value()?;
    let Value::Map(fields) = &mut root else {
        return Err("catalog root is not a map".into());
    };
    let Some(Value::Map(extension)) = fields.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
        return Err("runtime extension missing".into());
    };
    extension.insert(Key::Field(FieldId::new(2)?), Value::List(vec![plan]));
    seal_catalog(&mut root)?;
    Ok(Record::encode(&root)?.into_bytes())
}

pub(super) fn scalar_catalog_bytes(scalar: ScalarType) -> TestResult<Vec<u8>> {
    modeled_catalog(Some(scalar), false)
}

pub(super) fn typed_relationship_catalog(relations: Vec<Value>, dual: bool) -> TestResult<Vec<u8>> {
    let record = Record::from_bytes(modeled_catalog(None, dual)?)?;
    let mut root = inventory_core::record::ValueRef::Map(record.root()).to_value()?;
    let Value::Map(fields) = &mut root else {
        return Err("catalog root is not a map".into());
    };
    let Some(Value::Map(extension)) = fields.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
        return Err("runtime extension missing".into());
    };
    extension.insert(Key::Field(FieldId::new(4)?), Value::List(relations));
    seal_catalog(&mut root)?;
    Ok(Record::encode(&root)?.into_bytes())
}

pub(super) fn scalar_catalog_with_forged_text_root_relation(
    scalar: ScalarType,
) -> TestResult<Vec<u8>> {
    let record = Record::from_bytes(scalar_catalog_bytes(scalar)?)?;
    let mut root = inventory_core::record::ValueRef::Map(record.root()).to_value()?;
    let Value::Map(mut text_metadata) = runtime_metadata(Some(ScalarType::Text), false)? else {
        return Err("text metadata is not a map".into());
    };
    let relations_key = Key::Field(FieldId::new(4)?);
    let relations = text_metadata
        .remove(&relations_key)
        .ok_or("text relationships missing")?;
    let Value::Map(fields) = &mut root else {
        return Err("catalog root is not a map".into());
    };
    let Value::Map(metadata) = fields
        .get_mut(&Key::Field(FieldId::new(60_000)?))
        .ok_or("runtime extension missing")?
    else {
        return Err("runtime extension is not a map".into());
    };
    metadata.insert(relations_key, relations);
    seal_catalog(&mut root)?;
    Ok(Record::encode(&root)?.into_bytes())
}
#[test]
fn given_non_text_scalar_root_when_text_relationship_is_forged_then_checked_catalog_rejects_coercion()
-> TestResult {
    for scalar in [ScalarType::Integer, ScalarType::Bool] {
        let bytes = scalar_catalog_with_forged_text_root_relation(scalar)?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(inventory_core::catalog::runtime::RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

fn modeled_catalog(scalar: Option<ScalarType>, dual: bool) -> TestResult<Vec<u8>> {
    let member = |id| {
        catalog_map([
            (CatalogKey::MemberId, Value::Int(id)),
            (CatalogKey::MemberTarget, Value::Int(-3)),
            (CatalogKey::MemberRequired, Value::Bool(false)),
        ])
    };
    let input = catalog_map([
        (CatalogKey::Name, Value::Text("Input".into())),
        (CatalogKey::Kind, Value::Int(14)),
        (
            CatalogKey::Members,
            Value::List(vec![member(2)?, member(3)?]),
        ),
    ])?;
    let output = catalog_map([
        (CatalogKey::Name, Value::Text("Thing".into())),
        (CatalogKey::Kind, Value::Int(14)),
        (
            CatalogKey::Members,
            Value::List(if dual {
                vec![member(1)?, member(4)?, member(5)?]
            } else {
                vec![member(1)?]
            }),
        ),
    ])?;
    let shapes = vec![input, output];
    let shape_bytes = encode_list(&shapes)?;
    let shape_offsets = ListRef::from_bytes(&shape_bytes)?
        .encoded_item_offsets()?
        .iter()
        .flat_map(|offset| offset.to_be_bytes())
        .collect();
    let output = match scalar {
        None => 1,
        Some(ScalarType::Bool) => -2,
        Some(ScalarType::Integer) => -7,
        Some(ScalarType::Text) => -3,
        Some(_) => return Err("unsupported fixture scalar".into()),
    };
    let operation = catalog_map([
        (CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)),
        (CatalogKey::Input, Value::Int(0)),
        (CatalogKey::Output, Value::Int(output)),
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
        (CatalogKey::Shapes, Value::List(shapes)),
        (CatalogKey::ShapeOffsets, Value::Bytes(shape_offsets)),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let (names, offsets) = field_registry_values();
    let mut root = catalog_map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (
            CatalogKey::ModelsCommit,
            Value::Text("synthetic-example-fixture".into()),
        ),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())]),
        ),
    ])?;
    let Value::Map(fields) = &mut root else {
        return Err("catalog root is not a map".into());
    };
    fields.insert(
        Key::Field(FieldId::new(60_000)?),
        runtime_metadata(scalar, dual)?,
    );
    seal_catalog(&mut root)?;
    Ok(Record::encode(&root)?.into_bytes())
}

fn seal_catalog(root: &mut Value) -> TestResult {
    let Value::Map(fields) = root else {
        return Err("catalog root is not a map".into());
    };
    let mut base = fields.clone();
    base.remove(&Key::Field(FieldId::new(60_000)?));
    let model_hash = sha2::Sha256::digest(Record::encode(&Value::Map(base))?.as_bytes());
    let extension = fields
        .get_mut(&Key::Field(FieldId::new(60_000)?))
        .ok_or("runtime extension missing")?;
    let encoded = Record::encode(extension)?;
    let reviewed = compiled_inputs_sha256(encoded.root())?;
    // This is an explicitly synthetic Example fixture, not an upstream source receipt.
    let source_hash = sha2::Sha256::digest(include_bytes!("support.rs"));
    let Value::Map(extension) = extension else {
        return Err("runtime extension is not a map".into());
    };
    extension.insert(
        Key::Field(FieldId::new(7)?),
        wire_map([
            (1, Value::Int(1)),
            (
                2,
                wire_map([(1, Value::Int(1)), (2, Value::Bytes(model_hash.to_vec()))])?,
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

fn runtime_metadata(scalar: Option<ScalarType>, dual: bool) -> TestResult<Value> {
    let path = if scalar.is_some() {
        Vec::new()
    } else {
        vec![Value::Int(1)]
    };
    let scalar_id = match scalar {
        Some(ScalarType::Bool) => 0,
        Some(ScalarType::Integer) => 1,
        None | Some(ScalarType::Text) => 3,
        Some(_) => return Err("unsupported fixture scalar".into()),
    };
    let column = wire_map([(1, Value::List(path.clone())), (2, Value::Int(scalar_id))])?;
    let columns = if dual {
        vec![
            column,
            wire_map([(1, Value::List(vec![Value::Int(4)])), (2, Value::Int(3))])?,
        ]
    } else {
        vec![column]
    };
    let schema = wire_map([
        (1, Value::Text("AWS::Example::Thing".into())),
        (2, Value::Text("thing_table".into())),
        (3, Value::List(columns)),
        (4, Value::Null),
    ])?;
    let coverage = wire_map([
        (1, Value::Null),
        (2, Value::Text("AWS::Example::Thing".into())),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])?;
    let plan = callable_plan(scalar.is_some())?;
    let relations = if scalar.is_none() || scalar == Some(ScalarType::Text) {
        let rules = [
            (2, "references", path.clone()),
            (3, "depends-on", path.clone()),
        ];
        rules
            .into_iter()
            .chain(dual.then_some((3, "depends-on", vec![Value::Int(4)])))
            .map(|(source, relation, target_path)| {
                wire_map([
                    (1, Value::Text("AWS::Example::Thing".into())),
                    (2, Value::List(vec![Value::Int(source)])),
                    (3, Value::Text("AWS::Example::Thing".into())),
                    (4, Value::List(target_path)),
                    (5, Value::Text(relation.into())),
                    (6, Value::Int(0)),
                    (10, Value::Int(1)),
                ])
            })
            .collect::<TestResult<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let decision = wire_map([
        (1, Value::Text("example".into())),
        (2, Value::Text("ListThings".into())),
        (3, Value::Int(0)),
    ])?;
    wire_map([
        (1, Value::Int(2)),
        (2, Value::List(vec![plan])),
        (3, Value::List(vec![schema])),
        (4, Value::List(relations)),
        (5, Value::List(vec![coverage])),
        (6, Value::List(vec![decision])),
    ])
}

fn callable_plan(scalar: bool) -> TestResult<Value> {
    wire_map([
        (1, Value::Text("example".into())),
        (2, Value::Text("ListThings".into())),
        (3, Value::Text("AWS::Example::Thing".into())),
        (4, Value::Int(0)),
        (5, wire_map([(1, Value::Int(0))])?),
        (
            6,
            if scalar {
                wire_map([
                    (1, Value::Int(2)),
                    (2, Value::List(vec![Value::List(vec![])])),
                ])?
            } else {
                wire_map([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?
            },
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
                (4, Value::Text("fixture".into())),
                (5, Value::Int(0)),
                (6, Value::Int(0)),
                (7, Value::Int(0)),
                (8, Value::Int(0)),
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
    (
        Value::Bytes(b"NameReferenceValueDependencyValueAliasRevision".to_vec()),
        Value::Bytes(
            [0_u32, 4, 4, 18, 18, 33, 33, 38, 38, 46]
                .into_iter()
                .flat_map(u32::to_be_bytes)
                .collect(),
        ),
    )
}
