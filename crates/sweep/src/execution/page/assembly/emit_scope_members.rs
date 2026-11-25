use crate::consumer_fixture::TestResult;
use crate::{
    call,
    execution::page::{
        PageRequest,
        assembly::{self, Context},
        consumer_index::ConsumerIndex,
        cursor, preparation,
    },
};
use catalog_tool::{classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::catalog::runtime::CallablePlan;
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeProtocol},
    inventory::{
        AccountId, DiscoveryContext, OperationId, Region, RegionId, ResourceId, ResourceKey,
        ScopeKey, ScopeOutcome, ServiceId,
    },
    record::Record,
};
use inventory_engine::{
    inventory::{
        BinaryStore, DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, SchemaRegistry, SweepId,
        WorkLease, WorkSpec,
    },
    limiter::{ApiExecutor, ExecutorConfig},
};
use std::{collections::BTreeMap, path::Path};

const ACCOUNT: &str = "123456789012";
const REGION: &str = "us-east-1";
const ROLE_ARN: &str = "arn:aws:iam::123456789012:role/collector";
const IAM_DATE: &str = "2010-05-08";

#[tokio::test]
async fn given_role_when_two_complete_misses_then_sqlite_expires() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (_models, bytes) = fixture_catalog()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store =
        BinaryStore::create(&directory.path().join("scope-members.sqlite"), registry).await?;
    verify_scope_member_lifecycle(&store, &catalog).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL pointing at an empty test database"]
async fn given_role_when_two_complete_misses_then_postgres_expires() -> TestResult {
    let url = std::env::var("INVENTORY_POSTGRES_URL")?;
    let (_models, bytes) = fixture_catalog()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = PgStore::create(
        &DatabaseUrl::parse(&url)?,
        registry,
        PgWriteConfig::default(),
    )
    .await?;
    let result = verify_scope_member_lifecycle(&store, &catalog).await;
    store.close().await;
    result
}

async fn verify_scope_member_lifecycle<S: InventoryStore>(
    store: &S,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult {
    let list = catalog
        .lookup("iam", "ListRoles")
        .ok_or("ListRoles plan absent")?;
    let describe = catalog
        .lookup("iam", "GetRole")
        .ok_or("GetRole plan absent")?;
    let account = AccountId::parse(ACCOUNT)?;
    let home = RegionId::parse(REGION)?;
    let consumers = ConsumerIndex::build(catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = Context {
        catalog,
        account: &account,
        home: &home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 20,
    };
    let (baseline, key) = run_work_only_listing(store, &context, list, &account, &home).await?;
    describe_role(store, &context, describe, &baseline, &home).await?;
    assert!(store.projection_resource(&key).await?.is_some());
    complete_empty_listing(
        store,
        &context,
        list,
        &account,
        &home,
        "work-only-miss-one",
        ScopeOutcome::Complete,
    )
    .await?;
    assert!(store.projection_resource(&key).await?.is_some());
    complete_empty_listing(
        store,
        &context,
        list,
        &account,
        &home,
        "work-only-incomplete",
        ScopeOutcome::Partial,
    )
    .await?;
    assert!(store.projection_resource(&key).await?.is_some());
    complete_empty_listing(
        store,
        &context,
        list,
        &account,
        &home,
        "work-only-miss-two",
        ScopeOutcome::Complete,
    )
    .await?;
    assert!(store.projection_resource(&key).await?.is_none());
    Ok(())
}

async fn run_work_only_listing<S: InventoryStore>(
    store: &S,
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    account: &AccountId,
    home: &RegionId,
) -> TestResult<(SweepId, ResourceKey)> {
    let sweep = SweepId::new("work-only-baseline".to_owned())?;
    let listed = list_role(&sweep, plan, account)?;
    store.seed_works(std::slice::from_ref(&listed)).await?;
    let lease = claim(store, &sweep).await?;
    let request = PageRequest {
        record: Record::from_bytes(lease.bindings().to_vec())?,
        region: home.clone(),
    };
    let parts = assembly::prepare(
        context,
        &lease,
        plan,
        cursor::Cursor::default(),
        response(plan, roles_page())?,
        &request,
        &DiscoveryContext::empty(),
    )
    .await?;
    store
        .commit_page(super::super::page_commit(lease, parts, true)?)
        .await?;
    let key = ResourceKey {
        account: account.clone(),
        region: Region::Global,
        kind: inventory_core::inventory::Kind::parse("AWS::IAM::Role")?,
        id: ResourceId::parse(ROLE_ARN)?,
    };
    assert_eq!(store.projection_resources(None, 1000).await?.len(), 0);
    Ok((sweep, key))
}

async fn describe_role<S: InventoryStore>(
    store: &S,
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    sweep: &SweepId,
    home: &RegionId,
) -> TestResult {
    let lease = claim(store, sweep).await?;
    assert_eq!(lease.scope().operation().as_str(), "GetRole");
    let request = PageRequest {
        record: Record::from_bytes(lease.bindings().to_vec())?,
        region: home.clone(),
    };
    let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
    let parts = assembly::prepare(
        context,
        &lease,
        plan,
        cursor::Cursor::default(),
        response(plan, described_role())?,
        &request,
        &ancestry,
    )
    .await?;
    store
        .commit_page(super::super::page_commit(lease, parts, true)?)
        .await?;
    Ok(())
}

async fn complete_empty_listing<S: InventoryStore>(
    store: &S,
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    account: &AccountId,
    home: &RegionId,
    sweep_name: &str,
    outcome: ScopeOutcome,
) -> TestResult {
    let sweep = SweepId::new(sweep_name.to_owned())?;
    let work = list_role(&sweep, plan, account)?;
    store.seed_works(std::slice::from_ref(&work)).await?;
    let lease = claim(store, &sweep).await?;
    if outcome == ScopeOutcome::Partial {
        store
            .finish_scope(lease, outcome, crate::execution::now_ms()?, None)
            .await?;
        return Ok(());
    }
    let request = PageRequest {
        record: Record::from_bytes(lease.bindings().to_vec())?,
        region: home.clone(),
    };
    let parts = assembly::prepare(
        context,
        &lease,
        plan,
        cursor::Cursor::default(),
        response(plan, empty_roles_page())?,
        &request,
        &DiscoveryContext::empty(),
    )
    .await?;
    store
        .commit_page(super::super::page_commit(lease, parts, true)?)
        .await?;
    Ok(())
}

fn list_role(
    sweep: &SweepId,
    plan: &CallablePlan<'_>,
    account: &AccountId,
) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(
        account.clone(),
        Region::Global,
        ServiceId::parse("iam")?,
        OperationId::parse("ListRoles")?,
        None,
    )?;
    let bindings = crate::input::root_bindings(plan, Some(account), Some(&Region::Global))?;
    Ok(WorkSpec::new(
        sweep.clone(),
        scope,
        None,
        bindings.into_bytes(),
    )?)
}

async fn claim<S: InventoryStore>(store: &S, sweep: &SweepId) -> TestResult<WorkLease> {
    store
        .claim(sweep, crate::execution::now_ms()?, 3_600_000)
        .await?
        .ok_or_else(|| "sweep work was not queued".into())
}

fn fixture_catalog() -> TestResult<(tempfile::TempDir, Vec<u8>)> {
    let root = tempfile::tempdir()?;
    let source = root.path();
    let models = source.join("models");
    crate::pinned_model::install(&models, "iam", IAM_DATE, &["ListRoles", "GetRole"])?;
    let policies = work_only_iam_policy()?;
    let checked = overlay::check_policies(source, &[policies], &DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(source, &fields, "sweep-scope-members")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    let endpoint = fixture_endpoint()?;
    let endpoints = BTreeMap::from([(String::from("iam"), endpoint)]);
    let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
    Ok((root, bytes))
}

fn work_only_iam_policy() -> TestResult<overlay::ServiceOverlay> {
    let embedded = overlay::embedded()?;
    let service = embedded
        .iter()
        .find(|service| service.service() == "iam")
        .ok_or("IAM embedded overlay absent")?;
    let mut list = service
        .operations()
        .get("ListRoles")
        .cloned()
        .ok_or("ListRoles overlay absent")?;
    list.emission = overlay::Emission::WorkOnly;
    let describe = service
        .operations()
        .get("GetRole")
        .cloned()
        .ok_or("GetRole overlay absent")?;
    Ok(overlay::ServiceOverlay::new(
        "iam",
        vec![("ListRoles", list), ("GetRole", describe)],
    )?)
}

fn fixture_endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = serde_json::json!({
        "authSchemes": [{
            "name": "sigv4",
            "signingName": "iam",
            "signingRegion": "us-east-1"
        }]
    });
    let document = serde_json::json!({
        "version": "1.0",
        "parameters": {"Region": {"type": "string"}},
        "rules": [{
            "type": "endpoint",
            "conditions": [],
            "endpoint": {
                "url": "https://fixture.example.test",
                "properties": properties
            }
        }]
    });
    let rules = endpoints::RuleSet::parse(Path::new("sweep-test-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(serde_json::json!([{
        "params": {"Region": "us-east-1"},
        "expect": {
            "endpoint": {
                "url": "https://fixture.example.test",
                "properties": properties
            }
        }
    }]))?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("iam"))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}

fn response(plan: &CallablePlan<'_>, body: &str) -> TestResult<call::DecodedResponse> {
    let protocol = match plan.protocol() {
        RuntimeProtocol::AwsQuery => aws_wire::request::Protocol::AwsQuery,
        _ => return Err("IAM fixture plan did not use Query protocol".into()),
    };
    let body = body.as_bytes();
    Ok(call::DecodedResponse {
        http_version: http::Version::HTTP_11,
        record: aws_wire::response::decode(aws_wire::response::ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol,
            body,
            headers: &http::HeaderMap::new(),
            status: http::StatusCode::OK,
            limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
        })?,
        headers: http::HeaderMap::new(),
    })
}

fn roles_page() -> &'static str {
    "<ListRolesResponse><ListRolesResult><Roles><member><Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>%7B%22Statement%22%3A%5B%5D%7D</AssumeRolePolicyDocument></member></Roles><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>"
}

fn described_role() -> &'static str {
    "<GetRoleResponse><GetRoleResult><Role><Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>%7B%22Statement%22%3A%5B%5D%7D</AssumeRolePolicyDocument></Role></GetRoleResult></GetRoleResponse>"
}

fn empty_roles_page() -> &'static str {
    "<ListRolesResponse><ListRolesResult><Roles/><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>"
}
