use super::super::{IdentityContext, item_records, path_value, resource_id, tag_observation};
use aws_wire::{request::Protocol, response::{DecodeLimits, ResponseContext, decode}};
use catalog_tool::{classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{BindingSource, CallablePlan, RuntimeCatalog},
    inventory::{AccountId, Observation, OperationId, Region, RegionId, ResourceKey, ScopeKey, ServiceId},
    record::{Exclusions, Record},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::json;
use std::{collections::BTreeMap, error::Error, fs, path::{Path, PathBuf}};

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub(crate) fn compiled(service: &str, version: &str, operations: &[&str], kind: &str) -> TestResult<Vec<u8>> {
    compiled_with_operation(service, version, operations, kind, |_, _| {})
}

pub(crate) fn compiled_with_operation(
    service: &str, version: &str, operations: &[&str], kind: &str,
    mutate: impl Fn(&str, &mut overlay::OperationOverlay),
) -> TestResult<Vec<u8>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = tempfile::tempdir()?;
    crate::sweep_runtime::pinned_model::install(&source.path().join("models"), service, version, operations)?;
    let embedded = overlay::embedded()?;
    let service_policy = embedded.iter().find(|policy| policy.service() == service).ok_or("service absent")?;
    let policies = operations.iter().map(|name| {
        let mut policy = service_policy.operations().iter().find(|(key, _)| key.as_str() == *name)
            .map(|(_, policy)| policy.clone()).ok_or("operation absent")?;
        mutate(name, &mut policy);
        Ok((*name, policy))
    }).collect::<TestResult<Vec<_>>>()?;
    let policies = [overlay::ServiceOverlay::new(service, policies)?];
    let checked = overlay::check_policies(source.path(), &policies, &DenyList::builtin())?;
    let registry = smithy::Registry::parse(&fs::read_to_string(root.join("catalog/field-ids.txt"))?)?;
    let base = smithy::build_catalog(source.path(), &registry, "lp-extraction-regression")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    let endpoints = BTreeMap::from([(service.to_owned(), endpoint(service)?)]);
    let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let first = operations.first().ok_or("fixture needs an operation")?;
    assert_eq!(runtime.lookup(service, first).ok_or("compiled operation absent")?.kind().as_str(), kind);
    Ok(bytes)
}

fn endpoint(service: &str) -> TestResult<endpoints::VerifiedEndpoint> {
    // Only transport is a local fixture: all policy, model, decoder and identity code is production.
    let url = format!("https://{service}.test");
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":service,"signingRegion":"us-east-1"}]});
    let rules = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},
        "rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":url,"properties":properties}}]});
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},
        "expect":{"endpoint":{"url":url,"properties":properties}}}]))?;
    endpoints::RuleSet::parse(Path::new("lp-fixture-endpoint.json"), &rules)?.run_cases(&cases)?
        .verified_endpoint(Some(service)).map_err(|error| format!("fixture endpoint failed: {error:?}").into())
}

pub(crate) fn rows(plan: &CallablePlan<'_>, response: serde_json::Value) -> TestResult<Vec<Record>> {
    let bytes = serde_json::to_vec(&response)?;
    let headers = http::HeaderMap::new();
    let protocol = if plan.service_name() == "license-manager" { Protocol::AwsJson11 } else { Protocol::RestJson1 };
    let decoded = decode(ResponseContext {
        service: plan.service(), operation: plan.operation(), protocol, body: &bytes,
        headers: &headers, status: http::StatusCode::OK, limits: DecodeLimits::default(),
    })?;
    Ok(item_records(&decoded, plan.items())?)
}

pub(super) fn key(plan: &CallablePlan<'_>, row: &Record, region: &str) -> TestResult<ResourceKey> {
    let account = AccountId::parse("123456789012")?;
    let request_region = RegionId::parse(region)?;
    let id = resource_id(row, plan.identity(), Some(IdentityContext { account: &account, region: &request_region }))?;
    Ok(ResourceKey { account, region: Region::Regional(request_region), kind: plan.kind().clone(), id })
}

pub(super) fn discovered_input(plan: &CallablePlan<'_>, parent: &Record) -> TestResult<Record> {
    let binding = plan.bindings().first().ok_or("detail binding absent")?;
    let BindingSource::Operation { path, .. } = &binding.source else {
        return Err("detail must bind discovered parent output".into());
    };
    let value = path_value(parent.root(), path)?.ok_or("discovered ARN absent")?.to_value()?;
    // The detail request is dependent: root bindings correctly refuse it, so only the discovered parent field is bound.
    let base = Record::encode(&inventory_core::record::Value::Map(BTreeMap::new()))?;
    Ok(crate::sweep_runtime::input::merge_input(&base, &[
        crate::sweep_runtime::input::InputField { path: binding.path.clone(), value },
    ])?)
}

#[tracing::instrument(skip_all)]
async fn persist_snapshot(store: &BinaryStore, plan: &CallablePlan<'_>, input: &Record, row: Record, sweep: &str, at: i64) -> TestResult {
    let owner = key(plan, &row, "us-east-1")?;
    let tags = plan.tags().map(|plan| tag_observation(&row, &owner, plan)).transpose()?.into_iter().collect();
    let observation = Observation::new(owner.clone(), row, &Exclusions::none(), None)?;
    let parent = crate::sweep_runtime::input::requires_parent(plan).then_some(owner.clone());
    let scope = ScopeKey::new(owner.account.clone(), owner.region.clone(),
        ServiceId::parse(plan.service_name())?, OperationId::parse(plan.operation_name())?, parent)?;
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, input.clone().into_bytes())?).await?;
    let lease = store.claim(&sweep, at.saturating_sub(1), 100).await?.ok_or("detail lease absent")?;
    store.commit_page(PageCommit {
        lease, observations: vec![observation], edges: vec![], unresolved: vec![], tags,
        next_token: None, continuation_digest: None, children: vec![], done: true, observed_at_ms: at,
    }).await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn durable_snapshot(database: &Path) -> TestResult<(i64, Option<String>)> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(database).read_only(true),
    ).await?;
    let state = sqlx::query_as("SELECT (SELECT COUNT(*) FROM resources),(SELECT GROUP_CONCAT(tag_key || '=' || tag_value) FROM tags)")
        .fetch_one(&pool).await?;
    pool.close().await;
    Ok(state)
}

#[tracing::instrument(skip_all)]
pub(super) async fn assert_snapshot(store: &BinaryStore, database: &Path, plan: &CallablePlan<'_>, input: &Record, snapshot: (&str, i64, serde_json::Value, Option<&str>)) -> TestResult {
    let (sweep, at, response, expected) = snapshot;
    let row = rows(plan, response)?.into_iter().next().ok_or("detail row absent")?;
    persist_snapshot(store, plan, input, row, sweep, at).await?;
    assert_eq!(durable_snapshot(database).await?, (1, expected.map(str::to_owned)));
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn listed_store(bytes: &[u8], runtime: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>, summary: Record) -> TestResult<(tempfile::TempDir, BinaryStore, PathBuf)> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let registry = SchemaRegistry::new(bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&database, registry).await?;
    let input = crate::sweep_runtime::input::root_bindings(plan, None, None)?;
    persist_snapshot(&store, plan, &input, summary, "license-summary", 1).await?;
    Ok((directory, store, database))
}
