use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{catalog::runtime::{BindingSource, PlanScope}, inventory::{Observation, ResourceKey}, record::Value};
use std::{collections::BTreeMap, fs, path::Path, sync::LazyLock};

// These tests intentionally require the pinned model checkout used by catalog compilation.
// Missing model inputs fail the scenario; they never silently skip resource coverage.
const SERVICES: &[(&str, &str, &[&str])] = &[
    ("elasticsearch-service", "2015-01-01", &["ListDomainNames"]),
    ("opensearch", "2021-01-01", &["ListDomainNames"]),
    ("databrew", "2017-07-25", &["ListJobs", "ListProjects", "ListRulesets", "ListSchedules"]),
    ("database-migration-service", "2016-01-01", &["DescribeReplicationInstances", "DescribeInstanceProfiles"]),
    ("ecr", "2015-09-21", &["DescribeRepositories", "DescribeImages"]),
    ("datazone", "2018-05-10", &["ListDomains", "ListProjects", "ListEnvironments"]),
    ("iot", "2015-05-28", &["ListSecurityProfiles", "ListCustomMetrics", "ListDimensions"]),
    ("iottwinmaker", "2021-11-29", &["ListWorkspaces", "ListEntities"]),
    ("greengrassv2", "2020-11-30", &["ListDeployments"]),
    ("global-accelerator", "2018-08-08", &["ListAccelerators", "ListCustomRoutingAccelerators", "ListCrossAccountAttachments", "DescribeAccelerator", "ListTagsForResource", "ListListeners", "ListEndpointGroups", "ListCustomRoutingListeners", "ListCustomRoutingEndpointGroups"]),
    ("iam", "2010-05-08", &["ListRoles", "GetRole", "ListRoleTags"]),
    ("kafka", "2018-11-14", &["ListClustersV2"]),
    ("docdb", "2014-10-31", &["DescribeDBClusters"]),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(in crate::sweep_runtime) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned production-policy fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/aws-models/models");
    let embedded = overlay::embedded()?;
    let policies = SERVICES.iter().map(|(service, date, operations)| {
        let relative = format!("{service}/service/{date}/{service}-{date}.json");
        let destination = source.join("models").join(&relative);
        fs::create_dir_all(destination.parent().ok_or("model parent absent")?)?;
        fs::copy(pinned.join(&relative), destination)?;
        let policy = embedded.iter().find(|policy| policy.service() == *service)
            .ok_or("production service overlay absent")?;
        let selected = operations.iter().map(|operation| {
            let key = overlay::OperationName::new(operation)?;
            Ok((*operation, policy.operations().get(&key).ok_or("production operation absent")?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        Ok(overlay::ServiceOverlay::new(service, selected)?)
    }).collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}

pub(in crate::sweep_runtime) fn plan<'r, 'a>(catalog: &'r RuntimeCatalog<'a>, service: &str, operation: &str) -> TestResult<&'r CallablePlan<'a>> {
    catalog.lookup(service, operation).ok_or_else(|| format!("missing production plan {service}.{operation}").into())
}

pub(in crate::sweep_runtime) fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{request::Protocol, response::{DecodeLimits, ResponseContext}};
    use inventory_core::catalog::runtime::RuntimeProtocol;
    let protocol = match plan.protocol() {
        RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        RuntimeProtocol::AwsQuery => Protocol::AwsQuery,
        _ => return Err("unexpected policy fixture protocol".into()),
    };
    let headers = http::HeaderMap::new();
    let context = ResponseContext { service: plan.service(), operation: plan.operation(), protocol,
        body, headers: &headers, status: http::StatusCode::OK, limits: DecodeLimits::default() };
    Ok(if matches!(protocol, Protocol::AwsQuery) {
        aws_wire::response::xml::decode(context)?
    } else {
        aws_wire::response::json::decode(context)?
    })
}

pub(in crate::sweep_runtime) fn rows(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Vec<Record>> {
    Ok(item_records(&decode(plan, body)?, plan.items())?)
}

pub(super) fn request(plan: &CallablePlan<'_>, fields: &[input::InputField]) -> TestResult<http::Request<bytes::Bytes>> {
    let account = AccountId::parse("123456789012")?;
    let region = Region::Regional(RegionId::parse("us-east-1")?);
    let base = if input::requires_parent(plan) {
        Record::encode(&Value::Map(BTreeMap::new()))?
    } else { input::root_bindings(plan, Some(&account), Some(&region))? };
    let endpoint = plan.endpoint(&RegionId::parse("us-east-1")?)?;
    Ok(super::super::super::request::encode(plan, &endpoint.url, &base, fields)?)
}

pub(super) fn parent_fields(child: &CallablePlan<'_>, parent: &CallablePlan<'_>, row: &Record) -> TestResult<Vec<input::InputField>> {
    child.bindings().iter().map(|binding| {
        let BindingSource::Operation { service, operation, path } = &binding.source else {
            return Err("expected reviewed operation parent binding".into());
        };
        if service.as_ref() != parent.service_name() || operation.as_ref() != parent.operation_name() {
            return Err("binding selected wrong parent operation".into());
        }
        let value = path_value(row.root(), path)?.ok_or("required parent value absent")?.to_value()?;
        Ok(input::InputField { path: binding.path.clone(), value })
    }).collect()
}

pub(in crate::sweep_runtime) fn observations(plan: &CallablePlan<'_>, rows: Vec<Record>) -> TestResult<Vec<Observation>> {
    rows.into_iter().map(|row| {
        let key = ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: if plan.scope() == PlanScope::Account { Region::Global }
                else { Region::Regional(RegionId::parse("us-east-1")?) },
            kind: super::super::super::resource_projection::classify(plan, &row)?,
            id: resource_id(&row, plan.identity(), None)?,
        };
        let projected = super::super::super::resource_projection::project(plan, &row)?;
        Ok(Observation::new(key, projected, plan.hash_exclusions(), None)?
            .with_facet_precedence(plan.facet_precedence())?)
    }).collect()
}

pub(super) async fn persist(bytes: &[u8], plan: &CallablePlan<'_>, observations: Vec<Observation>) -> TestResult<Vec<(String, String, Vec<u8>)>> {
    use inventory_core::inventory::{OperationId, ScopeKey, ServiceId};
    use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
    let root = tempfile::tempdir()?;
    let path = root.path().join("inventory.db");
    let runtime = RuntimeCatalog::from_bytes(bytes)?;
    let mut unresolved = Vec::new();
    for observation in &observations {
        unresolved.extend(super::super::super::resource_projection::trust_references(&runtime, plan, observation)?);
    }
    let expected_references = i64::try_from(unresolved.len())?;
    let registry = SchemaRegistry::new(bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let sweep = SweepId::new("policy-consumer-scenario".to_owned())?;
    let region = if plan.scope() == PlanScope::Account { Region::Global } else { Region::Regional(RegionId::parse("us-east-1")?) };
    let scope = ScopeKey::new(AccountId::parse("123456789012")?, region,
        ServiceId::parse(plan.service_name())?, OperationId::parse(plan.operation_name())?, None)?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, Vec::new())?).await?;
    let lease = store.claim(&sweep, 1000, 10000).await?.ok_or("seeded work not claimable")?;
    store.commit_page(PageCommit { lease, observations, edges: Vec::new(), unresolved, tags: Vec::new(),
        next_token: None, continuation_digest: None, children: Vec::new(), done: true, observed_at_ms: 1001 }).await?;
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path).read_only(true)).await?;
    let saved = sqlx::query_as("SELECT kind,id,record FROM resource_facets ORDER BY kind,id").fetch_all(&reader).await?;
    let reference_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs").fetch_one(&reader).await?;
    assert_eq!(reference_count, expected_references);
    reader.close().await;
    Ok(saved)
}
