use cmdb_view::{GraphNode, PageRequest, ResourceFilter, TagFilter};
use inventory_core::{
    catalog::{CATALOG_VERSION, CatalogKey, runtime::RuntimeCatalog},
    inventory::{AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey,
        ScopeKey, ServiceId, Tag, TagCoverage, TagObservation},
    record::{Exclusions, FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use std::{collections::BTreeMap, error::Error, path::PathBuf, sync::Arc};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub const ACCOUNT: &str = "111111111111";
pub const FOREIGN: &str = "222222222222";
pub const REGION: &str = "us-east-1";
pub const KIND: &str = "AWS::Example::Thing";

pub struct Fixture {
    pub directory: tempfile::TempDir,
    pub path: PathBuf,
    pub store: BinaryStore,
}

pub async fn fixture() -> TestResult<Fixture> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let bytes = catalog()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    Ok(Fixture { directory, path, store })
}

pub fn key(account: &str, region: &str, id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey { account: AccountId::parse(account)?, region: Region::parse(region)?,
        kind: Kind::parse(KIND)?, id: ResourceId::parse(id)? })
}

pub fn node(account: &str, region: &str, id: &str) -> GraphNode {
    GraphNode { account: account.into(), region: region.into(), kind: KIND.into(), id: id.into() }
}

pub fn filter(account: &str, region: &str, tag_key: &str, value: Option<&str>) -> ResourceFilter {
    ResourceFilter { account: Some(account.into()), region: Some(region.into()), kind: Some(KIND.into()),
        text: None, tag: Some(TagFilter { key: tag_key.into(), value: value.map(str::to_owned) }) }
}

pub fn page(limit: usize) -> PageRequest { PageRequest { limit, after: None } }

pub fn observation(account: &str, region: &str, id: &str, name: &str) -> TestResult<Observation> {
    let record = Record::encode(&wire([(1, Value::Text(name.into()))])?)?;
    Ok(Observation::new(key(account, region, id)?, record, &Exclusions::none(), Some(Arc::from(name)))?)
}

pub fn tags(account: &str, region: &str, id: &str, coverage: TagCoverage, pairs: &[(&str, &str)]) -> TestResult<TagObservation> {
    Ok(TagObservation::new(key(account, region, id)?, coverage,
        pairs.iter().map(|(key, value)| Tag::new(*key, *value)).collect::<Result<Vec<_>, _>>()?)?)
}

pub async fn commit(store: &BinaryStore, sweep: &str, account: &str, region: &str,
    observations: Vec<Observation>, tags: Vec<TagObservation>, at: i64) -> TestResult {
    let scope = ScopeKey::new(AccountId::parse(account)?, Region::parse(region)?,
        ServiceId::parse("example")?, OperationId::parse("ListThings")?, None)?;
    let sweep = SweepId::new(sweep.into())?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, vec![1])?).await?;
    let lease = store.claim(&sweep, at, 100).await?.ok_or("missing consumer fixture lease")?;
    store.commit_page(PageCommit { lease, observations, tags, edges: vec![], unresolved: vec![],
        next_token: None, continuation_digest: None, children: vec![], done: true, observed_at_ms: at }).await?;
    Ok(())
}

fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(fields.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?))
}

fn fields(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    wire(entries.into_iter().map(|(key, value)| Ok((key.field()?.value(), value)))
        .collect::<TestResult<Vec<_>>>()?)
}

fn catalog() -> TestResult<Vec<u8>> {
    let plan = wire([
        (1, Value::Text("example".into())), (2, Value::Text("ListThings".into())),
        (3, Value::Text(KIND.into())), (4, Value::Int(0)),
        (5, wire([(1, Value::Int(0))])?),
        (6, wire([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?),
        (7, Value::List(vec![])),
        (8, wire([(1, wire([(1, Value::Int(0))])?), (2, Value::List(vec![]))])?),
        (9, Value::List(vec![])),
        (10, wire([(1, Value::Text("https://example.invalid".into())), (2, Value::Text(REGION.into())),
            (3, Value::Text("example".into())), (4, Value::Int(0)), (5, Value::Int(0))])?),
        (11, wire([(1, Value::Text("tag consumer writer fixture".into())),
            (2, Value::Text("test".into())), (3, Value::Text("2026-11-15".into()))])?),
        (12, wire([(1, Value::Int(0))])?), (13, Value::Int(5)),
    ])?;
    let schema = wire([(1, Value::Text(KIND.into())), (2, Value::Text("tag_consumer_records".into())),
        (3, Value::List(vec![wire([(1, Value::List(vec![Value::Int(1)])), (2, Value::Int(3))])?])),
        (4, Value::Null)])?;
    let coverage = wire([(1, Value::Null), (2, Value::Text(KIND.into())), (3, Value::Bool(true)),
        (4, Value::Null), (5, Value::Bool(true))])?;
    let decision = wire([(1, Value::Text("example".into())), (2, Value::Text("ListThings".into())), (3, Value::Int(0))])?;
    let runtime = wire([(1, Value::Int(2)), (2, Value::List(vec![plan])), (3, Value::List(vec![schema])),
        (4, Value::List(vec![])), (5, Value::List(vec![coverage])), (6, Value::List(vec![decision]))])?;
    let operation = fields([(CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)), (CatalogKey::Input, Value::Int(-1)), (CatalogKey::Output, Value::Int(-14))])?;
    let service = fields([(CatalogKey::Name, Value::Text("example".into())), (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#example".into())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".into())])),
        (CatalogKey::Operations, Value::List(vec![operation])), (CatalogKey::Shapes, Value::List(vec![])),
        (CatalogKey::ShapeOffsets, Value::Bytes(vec![])), (CatalogKey::Resources, Value::List(vec![]))])?;
    let offsets = (0_u32..8).flat_map(|index| [index * 2, (index + 1) * 2]).flat_map(u32::to_be_bytes).collect();
    let root = fields([(CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text("tag-consumer-contract".into())),
        (CatalogKey::FieldNames, Value::Bytes(b"a1a2a3a4a5a6a7a8".to_vec())),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())]))])?;
    let Value::Map(entries) = root else { return Err("catalog map required".into()); };
    super::seal::seal(entries, runtime)
}
