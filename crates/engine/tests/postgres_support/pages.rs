use super::{TestResult, registry};
use inventory_core::inventory::{
    AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey, ServiceId,
};
use inventory_core::record::{Exclusions, FieldId, Key, Record, Value};
use inventory_engine::inventory::{
    DatabaseUrl, PageCommit, PgStore, PgWriteConfig, SweepId, WorkLease, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::BTreeMap;

pub async fn run(check: impl AsyncFn(&PgStore, &PgPool, &DatabaseUrl) -> TestResult) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let pool = PgPool::connect(database.url()).await?;
    let (registry, _) = registry(1, None)?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let result = check(&store, &pool, &url).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

pub fn key(id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::Example::Kind0")?,
        id: ResourceId::parse(id)?,
    })
}

pub fn work(sweep: &str, parent: Option<ResourceKey>) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings0")?,
        parent,
    )?;
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        Vec::new(),
    )?)
}

pub async fn claim(store: &PgStore, sweep: &str, at: i64) -> TestResult<WorkLease> {
    Ok(store
        .claim(&SweepId::new(sweep.to_owned())?, at, 10_000)
        .await?
        .ok_or("missing lease")?)
}

pub async fn seed(store: &PgStore, sweep: &str) -> TestResult<WorkLease> {
    store.seed_works(&[work(sweep, None)?]).await?;
    claim(store, sweep, 1).await
}

pub fn observation(
    id: &str,
    fields: impl IntoIterator<Item = (u32, Value)>,
    masks: Exclusions,
) -> TestResult<Observation> {
    let mut fields = fields
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    fields.insert(Key::Field(FieldId::new(4)?), Value::Text(id.to_owned()));
    Ok(Observation::new(
        key(id)?,
        Record::encode(&Value::Map(fields))?,
        &masks,
        None,
    )?)
}

pub fn page(lease: WorkLease, observations: Vec<Observation>, token: u8, at: i64) -> PageCommit {
    PageCommit {
        lease,
        observations,
        edges: Vec::new(),
        unresolved: Vec::new(),
        tags: Vec::new(),
        next_token: Some(vec![token]),
        continuation_digest: Some(Sha256::digest([token]).into()),
        children: Vec::new(),
        done: false,
        observed_at_ms: at,
    }
}
