mod fixture;
mod quicksight;
mod children;
mod regional;
mod typed;
mod firstwave;
mod multi_emission;

use super::*;
use super::super::{assembly, consumer_index::ConsumerIndex, cursor, PageRequest};
use crate::sweep_runtime::execution::routing;
use inventory_core::catalog::runtime::{BindingSource, PlanScope, RuntimeCatalog};
use inventory_engine::{inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease}, limiter::{ApiExecutor, ExecutorConfig}};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type Wire = http::Request<bytes::Bytes>;

struct Scenario<'a, 'c> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'c>,
    executor: ApiExecutor,
}

impl<'a, 'c> Scenario<'a, 'c> {
    async fn new(catalog: &'a RuntimeCatalog<'c>, name: &str, home: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self { directory, store, sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?, home: RegionId::parse(home)?,
            consumers: ConsumerIndex::build(catalog)?, executor: ApiExecutor::new(ExecutorConfig::default())? })
    }

    async fn reopen(self, catalog: &RuntimeCatalog<'_>) -> TestResult<Self> {
        let Self { directory, store, sweep, account, home, consumers, executor } = self;
        drop(store);
        let store = BinaryStore::open_with_catalog(&directory.path().join("inventory.db"), catalog.raw_bytes()).await?;
        Ok(Self { directory, store, sweep, account, home, consumers, executor })
    }

    async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
        let region = if plan.scope() == PlanScope::Account { Region::Global } else { Region::Regional(self.home.clone()) };
        let scope = ScopeKey::new(self.account.clone(), region.clone(), ServiceId::parse(plan.service_name())?,
            OperationId::parse(plan.operation_name())?, None)?;
        let base = input::root_bindings(plan, Some(&self.account), Some(&region))?;
        if plan.branches().is_empty() {
            self.store.seed_work(WorkSpec::new(self.sweep.clone(), scope, None, base.into_bytes())?).await?;
        } else {
            for branch in plan.branches() {
                let fields = branch.bindings.iter().map(|binding| match &binding.source {
                    BindingSource::Fixed(value) => Ok(input::InputField { path: binding.path.clone(), value: crate::sweep_runtime::request::fixed_input::value(value)? }),
                    _ => Err("unexpected nonfixed branch input".into()),
                }).collect::<TestResult<Vec<_>>>()?;
                let branch_scope = scope.clone().with_branch(branch.discriminator.clone())?;
                self.store.seed_work(WorkSpec::new(self.sweep.clone(), branch_scope, None,
                    input::merge_input(&base, &fields)?.into_bytes())?).await?;
            }
        }
        Ok(())
    }

    fn request(&self, plan: &CallablePlan<'_>, lease: &WorkLease) -> TestResult<(Record, Wire)> {
        let region = routing::request_region(plan, lease.scope().region(), &self.home, lease.scope().branch())?;
        let endpoint = routing::resolve(plan, &region)?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let mut fields = cursor::Cursor::decode(lease.token())?.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField { path: size.path.clone(), value: Value::Int(i64::from(size.max)) });
        }
        let wire = crate::sweep_runtime::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    async fn step(&self, catalog: &RuntimeCatalog<'_>, respond: &mut impl FnMut(&CallablePlan<'_>, &WorkLease, &Wire) -> TestResult<Vec<u8>>) -> TestResult {
        let lease = self.store.claim(&self.sweep, 1001, 10_000).await?.ok_or("expected native page work lost")?;
        self.step_lease(catalog, lease, respond).await
    }

    async fn step_lease(&self, catalog: &RuntimeCatalog<'_>, lease: WorkLease,
        respond: &mut impl FnMut(&CallablePlan<'_>, &WorkLease, &Wire) -> TestResult<Vec<u8>>) -> TestResult {
        let plan = fixture::plan(catalog, lease.scope().service().as_str(), lease.scope().operation().as_str())?;
        let (request, wire) = self.request(plan, &lease)?;
        let response = crate::sweep_runtime::call::DecodedResponse {
            record: fixture::decode(plan, &respond(plan, &lease, &wire)?)?, headers: http::HeaderMap::new(),
        };
        let ancestry = super::super::preparation::ancestry(lease.context(), lease.scope().context())?;
        let request = PageRequest { record: request,
            region: routing::request_region(plan, lease.scope().region(), &self.home, lease.scope().branch())? };
        let context = assembly::Context { catalog, account: &self.account, home: &self.home,
            executor: &self.executor, consumers: &self.consumers, max_pages: 10 };
        let parts = assembly::prepare(&context, &lease, plan, cursor::Cursor::decode(lease.token())?, response, &request, &ancestry).await?;
        let done = parts.next.is_none();
        let (next_token, continuation_digest) = parts.next.map_or((None, None), |next| (Some(next.cursor.into_bytes()), next.continuation_digest));
        self.store.commit_page(PageCommit { lease, observations: parts.observations, children: parts.children,
            unresolved: parts.references, tags: parts.tags, edges: Vec::new(), next_token, continuation_digest, done, observed_at_ms: 1001 }).await?;
        Ok(())
    }

    async fn run(&self, catalog: &RuntimeCatalog<'_>, pages: usize,
        mut respond: impl FnMut(&CallablePlan<'_>, &WorkLease, &Wire) -> TestResult<Vec<u8>>) -> TestResult {
        for _ in 0..pages { self.step(catalog, &mut respond).await?; }
        assert!(self.store.claim(&self.sweep, 1001, 10_000).await?.is_none(), "unexpected extra or unfinished producer");
        Ok(())
    }

    async fn saved(&self) -> TestResult<Vec<(String, String, String)>> {
        let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(self.directory.path().join("inventory.db")).read_only(true)).await?;
        let rows = sqlx::query_as("SELECT kind,id,region FROM resources ORDER BY kind,id").fetch_all(&reader).await?;
        reader.close().await;
        Ok(rows)
    }

    async fn edges(&self) -> TestResult<Vec<(String, String, String, String, String)>> {
        let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(self.directory.path().join("inventory.db")).read_only(true)).await?;
        let edges = sqlx::query_as("SELECT from_kind,from_id,to_kind,to_id,relation FROM edges ORDER BY from_kind,from_id,to_kind,to_id,relation").fetch_all(&reader).await?;
        reader.close().await;
        Ok(edges)
    }
}

fn text<'a>(value: &'a Json, field: &str) -> TestResult<&'a str> {
    value.get(field).and_then(Json::as_str).ok_or_else(|| format!("native request omitted {field}").into())
}
fn body(wire: &Wire) -> TestResult<Json> {
    if wire.headers().get("smithy-protocol").is_some_and(|value| value == "rpc-v2-cbor") { return crate::sweep_runtime::test_cbor::cbor_to_json(wire.body()); }
    Ok(serde_json::from_slice(wire.body())?)
}
fn bytes(value: Json) -> TestResult<Vec<u8>> { Ok(serde_json::to_vec(&value)?) }
fn paginated(mut value: Json, token: &str, second: bool) -> TestResult<Vec<u8>> {
    if !second { value.as_object_mut().ok_or("native response not an object")?.insert(token.to_owned(), json!("page-two")); }
    bytes(value)
}
fn count(rows: &[(String, String, String)], kind: &str) -> usize { rows.iter().filter(|(saved, _, _)| saved == kind).count() }
