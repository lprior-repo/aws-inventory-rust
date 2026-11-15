mod catalog;
mod lattice;
mod mail;
mod mail_collision;
mod metadata;
mod workspaces;
mod waf;

use super::*;
use super::super::{assembly, PageRequest, consumer_index::ConsumerIndex, cursor, preparation};
use crate::sweep_runtime::{execution::routing, record};
use inventory_core::catalog::runtime::{RuntimeCatalog, PlanScope};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use serde_json::{Value as Json, json};
use std::{collections::BTreeMap, error::Error, sync::LazyLock};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
static FIELDS: LazyLock<Result<catalog_tool::smithy::Registry, String>> = LazyLock::new(|| {
    catalog_tool::smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))
        .map_err(|error| error.to_string())
});

struct Scenario<'a, 'catalog> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

impl<'a, 'catalog> Scenario<'a, 'catalog> {
    async fn new(catalog: &'a RuntimeCatalog<'catalog>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self { directory, store, sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?, home: RegionId::parse("us-east-1")?,
            consumers: ConsumerIndex::build(catalog)? })
    }

    async fn seed(&self, catalog: &RuntimeCatalog<'_>, service: &str, operations: &[&str]) -> TestResult {
        for operation in operations {
            let plan = catalog.lookup(service, operation).ok_or("seed plan absent")?;
            let region = if plan.scope() == PlanScope::Account { Region::Global } else { Region::Regional(self.home.clone()) };
            let scope = ScopeKey::new(self.account.clone(), region.clone(),
                ServiceId::parse(service)?, OperationId::parse(operation)?, None)?;
            let input = input::root_bindings(plan, Some(&self.account), Some(&region))?;
            self.store.seed_work(WorkSpec::new(self.sweep.clone(), scope, None, input.into_bytes())?).await?;
        }
        Ok(())
    }

    fn request(&self, plan: &CallablePlan<'_>, lease: &WorkLease) -> TestResult<(Record, http::Request<bytes::Bytes>)> {
        let region = routing::request_region(plan, lease.scope().region(), &self.home, lease.scope().branch())?;
        let endpoint = routing::resolve(plan, &region)?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let mut fields = cursor.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField { path: size.path.clone(), value: Value::Int(i64::from(size.max)) });
        }
        let wire = crate::sweep_runtime::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    async fn apply(&self, catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>, lease: WorkLease,
        request: &Record, body: Json) -> TestResult {
        self.apply_wire(catalog, plan, lease, request, &serde_json::to_vec(&body)?).await
    }

    async fn apply_wire(&self, catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>, lease: WorkLease,
        request: &Record, body: &[u8]) -> TestResult {
        let executor = inventory_engine::limiter::ApiExecutor::new(
            inventory_engine::limiter::ExecutorConfig::default())?;
        let context = assembly::Context { catalog, account: &self.account, home: &self.home,
            executor: &executor, consumers: &self.consumers, max_pages: 10 };
        let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
        let parts = assembly::prepare(&context, &lease, plan, cursor::Cursor::decode(lease.token())?,
            crate::sweep_runtime::call::DecodedResponse {
                record: decode(plan, body)?, headers: http::HeaderMap::new(),
            }, &PageRequest { record: request.clone(),
                region: routing::request_region(plan, lease.scope().region(), &self.home, lease.scope().branch())? },
            &ancestry).await?;
        let done = parts.next.is_none();
        let (next_token, continuation_digest) = parts.next.map_or((None, None), |advance| {
            (Some(advance.cursor.into_bytes()), advance.continuation_digest)
        });
        self.store.commit_page(PageCommit { lease, observations: parts.observations,
            unresolved: parts.references, tags: parts.tags, children: parts.children,
            edges: Vec::new(), next_token, continuation_digest, done, observed_at_ms: 1001 }).await?;
        Ok(())
    }

    async fn drain(mut self, catalog: &RuntimeCatalog<'_>, service: &str,
        mut respond: impl FnMut(&CallablePlan<'_>, &Record, &http::Request<bytes::Bytes>, bool) -> TestResult<Json>,
    ) -> TestResult<(Self, BTreeMap<String, usize>)> {
        let mut seen = BTreeMap::new();
        while let Some(lease) = self.store.claim(&self.sweep, 1000, 10000).await? {
            let operation = lease.scope().operation().as_str();
            let plan = catalog.lookup(lease.scope().service().as_str(), operation).ok_or("leased producer absent")?;
            assert!(plan.service_name() == service || matches!(plan.service_name(), "cloudwatch-logs" | "firehose"),
                "unexpected cross-service fixture producer");
            let (request, wire) = self.request(plan, &lease)?;
            let body = respond(plan, &request, &wire, lease.token().is_some())?;
            *seen.entry(operation.to_owned()).or_insert(0) += 1;
            assert!(seen.values().sum::<usize>() < 512, "fixture unexpectedly repeated work");
            self.apply(catalog, plan, lease, &request, body).await?;
            self = self.reopen().await?;
        }
        Ok((self, seen))
    }

    async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new()
                .filename(self.directory.path().join("inventory.db")).read_only(true)).await?)
    }

    async fn reopen(self) -> TestResult<Self> {
        let Self { directory, store, sweep, account, home, consumers } = self;
        drop(store);
        let store = BinaryStore::open(&directory.path().join("inventory.db")).await?;
        Ok(Self { directory, store, sweep, account, home, consumers })
    }

    async fn assert_ids(&self, kind: &str, expected: Vec<String>) -> TestResult {
        self.assert_ids_at(kind, "us-east-1", expected).await
    }

    async fn assert_ids_at(&self, kind: &str, region: &str, expected: Vec<String>) -> TestResult {
        let reader = self.reader().await?;
        let actual: Vec<(String,String,String,String)> = sqlx::query_as(
            "SELECT account,region,kind,id FROM resources WHERE kind=? ORDER BY id")
            .bind(kind).fetch_all(&reader).await?;
        reader.close().await;
        let mut expected: Vec<_> = expected.into_iter().map(|id|
            ("123456789012".to_owned(),region.to_owned(),kind.to_owned(),id)).collect();
        expected.sort();
        assert_eq!(actual, expected, "{kind} canonical ResourceKeys");
        Ok(())
    }

    async fn assert_edges(&self, kind: &str, relation: &str, expected: Vec<(String,String,String)>) -> TestResult {
        self.assert_edges_at(kind, relation, "us-east-1", "us-east-1", expected).await
    }

    async fn assert_edges_at(&self, kind: &str, relation: &str, from_region: &str, to_region: &str,
        expected: Vec<(String,String,String)>) -> TestResult {
        self.assert_edge_regions(kind,relation,from_region,expected.into_iter().map(|(from,kind,to)|
            (from,to_region.to_owned(),kind,to)).collect()).await
    }

    async fn assert_edge_regions(&self, kind: &str, relation: &str, from_region: &str,
        expected: Vec<(String,String,String,String)>) -> TestResult {
        let reader = self.reader().await?;
        let mut actual: Vec<(String,String,String,String,String,String,String,String,String,String)> =
            sqlx::query_as("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE from_kind=? AND relation=?")
                .bind(kind).bind(relation).fetch_all(&reader).await?;
        reader.close().await;
        let mut expected: Vec<_> = expected.into_iter().map(|(from,to_region,to_kind,to)|
            ("123456789012".to_owned(),from_region.to_owned(),kind.to_owned(),from,
             "123456789012".to_owned(),to_region.to_owned(),to_kind,to,relation.to_owned(),"reviewed-rule".to_owned())).collect();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{kind} {relation} resolved DAG edges");
        Ok(())
    }
}

fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{request::Protocol, response::{DecodeLimits, ResponseContext}};
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        inventory_core::catalog::runtime::RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        inventory_core::catalog::runtime::RuntimeProtocol::RestXml => Protocol::RestXml,
        _ => return Err("unexpected native protocol in T-Z fixture".into()),
    };
    Ok(aws_wire::response::decode(ResponseContext { service: plan.service(), operation: plan.operation(), protocol,
        body, headers: &http::HeaderMap::new(), status: http::StatusCode::OK, limits: DecodeLimits::default() })?)
}

fn text<'a>(record: &'a Record, name: &str) -> TestResult<&'a str> {
    let fields = FIELDS.as_ref().map_err(|error| error.clone())?;
    let path = [fields.id(name).ok_or("named input field absent")?];
    match record::path_value(record.root(), &path)? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err(format!("encoded input omitted {name}").into()),
    }
}

fn page(items: &str, row: Json, next: bool, token_name: &str) -> Json {
    let mut body = serde_json::Map::new();
    body.insert(items.to_owned(), Json::Array(vec![row]));
    if next { body.insert(token_name.to_owned(), json!("page-two")); }
    Json::Object(body)
}

fn correlated(parts: &[&str]) -> String {
    parts.iter().map(|part| format!("{}:{part}", part.len())).collect::<Vec<_>>().join("|")
}

fn assert_token(wire: &http::Request<bytes::Bytes>, name: &str, resumed: bool) -> TestResult {
    let expected = resumed.then_some("page-two");
    if wire.method() == http::Method::GET {
        let actual = wire.uri().query().unwrap_or_default().split('&')
            .find_map(|pair| pair.strip_prefix(&format!("{name}=")));
        assert_eq!(actual, expected);
    } else {
        let body: Json = if wire.body().is_empty() { Json::Null } else { serde_json::from_slice(wire.body())? };
        assert_eq!(body.get(name).and_then(Json::as_str), expected);
    }
    Ok(())
}
