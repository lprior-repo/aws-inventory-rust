#[path = "modeled_catalog.rs"]
mod modeled_catalog;
use cmdb_view::{GraphNode, PageRequest, ResourceFilter, TagFilter};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{
        AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey,
        ServiceId, Tag, TagCoverage, TagObservation,
    },
    record::{Exclusions, FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{
    DatabaseUrl, PageCommit, PgStore, PgWriteConfig, SchemaRegistry, SweepId, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use modeled_catalog::{Model, modeled_catalog};
use sqlx::PgPool;
use std::{collections::BTreeMap, error::Error, future::Future, sync::Arc, time::Duration};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub const ACCOUNT: &str = "111111111111";
pub const FOREIGN: &str = "222222222222";
pub const REGION: &str = "us-east-1";
pub const KIND: &str = "AWS::Example::Thing";

pub struct Fixture {
    database: TestDatabase,
    pub store: PgStore,
    pub pool: PgPool,
}

impl Fixture {
    pub fn url(&self) -> &str {
        self.database.url()
    }

    pub async fn reopen(self) -> TestResult<Self> {
        let url = DatabaseUrl::parse(self.database.url())?;
        self.store.close().await;
        let store = PgStore::open(&url, PgWriteConfig::default()).await?;
        Ok(Self { store, ..self })
    }

    pub async fn set_schema_version(&self, value: &[u8]) -> TestResult {
        sqlx::query("UPDATE inventory.metadata SET value=$1 WHERE key='schema_version'")
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn finish(self) -> TestResult {
        self.store.close().await;
        self.pool.close().await;
        self.database.finish().await?;
        Ok(())
    }
}

pub async fn fixture() -> TestResult<Fixture> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let bytes = catalog()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let pool = PgPool::connect(database.url()).await?;
    Ok(Fixture {
        database,
        store,
        pool,
    })
}

pub async fn bounded(test: impl Future<Output = TestResult>) -> TestResult {
    tokio::time::timeout(Duration::from_secs(120), test).await?
}

pub fn key(account: &str, region: &str, id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        kind: Kind::parse(KIND)?,
        id: ResourceId::parse(id)?,
    })
}

pub fn node(account: &str, region: &str, id: &str) -> GraphNode {
    GraphNode {
        account: account.into(),
        region: region.into(),
        kind: KIND.into(),
        id: id.into(),
    }
}

pub fn filter(account: &str, region: &str, tag_key: &str, value: Option<&str>) -> ResourceFilter {
    ResourceFilter {
        account: Some(account.into()),
        region: Some(region.into()),
        kind: Some(KIND.into()),
        text: None,
        tag: Some(TagFilter {
            key: tag_key.into(),
            value: value.map(str::to_owned),
        }),
    }
}

pub fn page(limit: usize) -> PageRequest {
    PageRequest { limit, after: None }
}

pub fn observation(account: &str, region: &str, id: &str, name: &str) -> TestResult<Observation> {
    let record = Record::encode(&wire([(1, Value::Text(name.into()))])?)?;
    Ok(Observation::new(
        key(account, region, id)?,
        record,
        &Exclusions::none(),
        Some(Arc::from(name)),
    )?)
}

pub fn tags(
    account: &str,
    region: &str,
    id: &str,
    coverage: TagCoverage,
    pairs: &[(&str, &str)],
) -> TestResult<TagObservation> {
    Ok(TagObservation::new(
        key(account, region, id)?,
        coverage,
        pairs
            .iter()
            .map(|(key, value)| Tag::new(*key, *value))
            .collect::<Result<Vec<_>, _>>()?,
    )?)
}

pub async fn commit(
    store: &PgStore,
    sweep: &str,
    account: &str,
    region: &str,
    observations: Vec<Observation>,
    tags: Vec<TagObservation>,
    at: i64,
) -> TestResult {
    let scope = ScopeKey::new(
        AccountId::parse(account)?,
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?;
    let sweep = SweepId::new(sweep.into())?;
    store
        .seed_works(&[WorkSpec::new(sweep.clone(), scope, None, vec![1])?])
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("missing consumer fixture lease")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            tags,
            edges: vec![],
            unresolved: vec![],
            next_token: None,
            continuation_digest: None,
            children: vec![],
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}

fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        fields
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<TestResult<BTreeMap<_, _>>>()?,
    ))
}

fn catalog() -> TestResult<Vec<u8>> {
    modeled_catalog(&Model {
        kind: KIND,
        region: REGION,
        table: "tag_consumer_records",
        commit: "tag-consumer-contract",
    })
}
