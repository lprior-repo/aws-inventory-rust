#[path = "../../../engine/tests/postgres_support/mod.rs"]
mod catalog;

use cmdb_view::{Database, GraphEdge, GraphNode};
use inventory_core::inventory::{
    AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey, ServiceId,
};
use inventory_core::record::{Exclusions, FieldId, Key, Record, Value};
use inventory_engine::inventory::{
    DatabaseUrl, PageCommit, PgStore, PgWriteConfig, SweepId, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::PgPool;
use std::collections::BTreeMap;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub struct Fixture {
    database: TestDatabase,
    pub store: PgStore,
    pub pool: PgPool,
}

impl Fixture {
    pub async fn new() -> TestResult<Self> {
        let database = TestDatabase::create(&admin_url()?).await?;
        let url = DatabaseUrl::parse(database.url())?;
        let (registry, _) = catalog::registry(2, None)?;
        let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
        let pool = PgPool::connect(database.url()).await?;
        let fixture = Self {
            database,
            store,
            pool,
        };
        fixture
            .commit("one", "111111111111", "global", 0, &["shared"], 10)
            .await?;
        fixture
            .commit("two", "111111111111", "us-east-1", 0, &["shared"], 20)
            .await?;
        fixture
            .commit("three", "222222222222", "eu-west-1", 0, &["shared"], 30)
            .await?;
        fixture
            .commit("four", "222222222222", "us-west-2", 1, &["shared"], 40)
            .await?;
        fixture
            .edge(&edge(
                &node("111111111111", "global", 0, "shared"),
                &node("111111111111", "us-east-1", 0, "shared"),
                "replica",
                "catalog",
            ))
            .await?;
        fixture
            .edge(&edge(
                &node("111111111111", "global", 0, "shared"),
                &node("222222222222", "eu-west-1", 0, "shared"),
                "replica",
                "catalog",
            ))
            .await?;
        Ok(fixture)
    }

    pub async fn reader(&self) -> TestResult<Database> {
        Ok(Database::open(self.database.url()).await?)
    }

    pub fn url(&self) -> &str {
        self.database.url()
    }

    pub async fn finish(self) -> TestResult {
        self.store.close().await;
        self.pool.close().await;
        self.database.finish().await?;
        Ok(())
    }

    pub async fn commit(
        &self,
        sweep: &str,
        account: &str,
        region: &str,
        kind: u32,
        ids: &[&str],
        at: i64,
    ) -> TestResult {
        let scope = ScopeKey::new(
            AccountId::parse(account)?,
            Region::parse(region)?,
            ServiceId::parse("example")?,
            OperationId::parse(&format!("ListThings{kind}"))?,
            None,
        )?;
        let sweep = SweepId::new(sweep.to_owned())?;
        self.store
            .seed_works(&[WorkSpec::new(sweep.clone(), scope, None, Vec::new())?])
            .await?;
        let lease = self
            .store
            .claim(&sweep, at, 10_000)
            .await?
            .ok_or("missing fixture lease")?;
        let observations = ids
            .iter()
            .map(|id| {
                let key = key(account, region, kind, id)?;
                Ok(Observation::new(
                    key,
                    record(id, at)?,
                    &Exclusions::none(),
                    None,
                )?)
            })
            .collect::<TestResult<Vec<_>>>()?;
        self.store
            .commit_page(PageCommit {
                lease,
                observations,
                edges: Vec::new(),
                unresolved: Vec::new(),
                tags: Vec::new(),
                next_token: None,
                continuation_digest: None,
                children: Vec::new(),
                done: true,
                observed_at_ms: at,
            })
            .await?;
        Ok(())
    }

    pub async fn edge(&self, edge: &GraphEdge) -> TestResult {
        sqlx::query("INSERT INTO inventory.edges VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
            .bind(&edge.from.account)
            .bind(&edge.from.region)
            .bind(&edge.from.kind)
            .bind(&edge.from.id)
            .bind(&edge.to.account)
            .bind(&edge.to.region)
            .bind(&edge.to.kind)
            .bind(&edge.to.id)
            .bind(&edge.relation)
            .bind(&edge.provenance)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

pub fn key(account: &str, region: &str, kind: u32, id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        kind: Kind::parse(&format!("AWS::Example::Kind{kind}"))?,
        id: ResourceId::parse(id)?,
    })
}

pub fn node(account: &str, region: &str, kind: u32, id: &str) -> GraphNode {
    GraphNode {
        account: account.to_owned(),
        region: region.to_owned(),
        kind: format!("AWS::Example::Kind{kind}"),
        id: id.to_owned(),
    }
}

pub fn root() -> GraphNode {
    node("111111111111", "global", 0, "shared")
}

pub fn edge(from: &GraphNode, to: &GraphNode, relation: &str, provenance: &str) -> GraphEdge {
    GraphEdge {
        from: from.clone(),
        to: to.clone(),
        relation: relation.to_owned(),
        provenance: provenance.to_owned(),
    }
}

pub fn record(id: &str, at: i64) -> TestResult<Record> {
    let fields = [(2, Value::Int(at)), (4, Value::Text(id.to_owned()))]
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    Ok(Record::encode(&Value::Map(fields))?)
}
