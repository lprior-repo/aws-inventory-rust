mod rest_position;
mod native_fixture;
mod owned_roots;

use native_fixture::{child_body, page_keys};
use owned_roots::filtered_roots;

use super::{TestResult, compile, cursor::Cursor, decode, resource_key};
use crate::sweep_runtime::{execution::page::{child, consumer_index::ConsumerIndex}, input, record, request};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId}, record::{Record, ValueRef}};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value, json};
use std::collections::BTreeSet;

struct ChildCase {
    operation: &'static str,
    kind: &'static str,
    items: &'static str,
    id: &'static str,
}

struct Family {
    service: &'static str,
    date: &'static str,
    root: &'static str,
    root_items: &'static str,
    root_key: &'static str,
    input_key: &'static str,
    token: &'static str,
    children: &'static [ChildCase],
}

fn text(value: &Record, fields: &catalog_tool::smithy::Registry, key: &str) -> TestResult<String> {
    match record::path_value(value.root(), &[fields.id(key).ok_or("modeled request member absent")?])? {
        Some(ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err(format!("required request text absent: {key}").into()),
    }
}

fn parent_id(family: &Family, suffix: &str) -> String {
    if family.service == "batch" { format!("arn:aws:batch:us-east-1:123456789012:job-queue/{suffix}") }
    else { format!("api-{suffix}") }
}


async fn consume_family(family: &Family) -> TestResult {
    // Given: two real parent rows and the embedded, pinned-model-compiled child plans.
    let excluded: &[&str] = match family.service {
        "api-gateway" => &["GetIntegration", "GetMethod", "GetModel", "GetModels", "GetExport", "GetSdk", "PutRestApi"],
        "apigatewayv2" => &["GetIntegrations", "GetModels", "CreateRoute"],
        "batch" => &["DescribeJobDefinitions", "DescribeJobs", "SubmitJob"],
        _ => return Err("unreviewed child fixture family".into()),
    };
    let operations = std::iter::once(family.root).chain(family.children.iter().map(|case| case.operation))
        .chain(excluded.iter().copied()).collect::<Vec<_>>();
    let compiled = compile(family.service, family.date, &operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for operation in excluded { assert!(catalog.lookup(family.service, operation).is_none(), "content or write plan became callable: {operation}"); }
    let consumers = ConsumerIndex::build(&catalog)?;
    let parent = catalog.lookup(family.service, family.root).ok_or("parent plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new(format!("{}-safe-children", family.service))?;
    let scope = ScopeKey::new(account.clone(), region.clone(), ServiceId::parse(family.service)?, OperationId::parse(family.root)?, None)?;
    let root_request = input::root_bindings(parent, Some(&account), Some(&region))?;
    let body = json!({family.root_items:[{family.root_key:parent_id(family,"a")},{family.root_key:parent_id(family,"b")}]});
    let response = decode(parent, &body)?;
    let context = child::Context::new(parent, &sweep, &account, &home, &consumers);
    let (observations, children) = child::policy_dfgk_tests::project_children(&catalog, &context, &scope, &root_request, &response, &DiscoveryContext::empty())?;
    assert_eq!(observations.len(), 2);
    let (root_kind, relation) = match family.service {
        "api-gateway" => ("AWS::ApiGateway::RestApi", "IN_REST_API"),
        "apigatewayv2" => ("AWS::ApiGatewayV2::Api", "IN_API"),
        "batch" => ("AWS::Batch::JobQueue", "IN_JOB_QUEUE"),
        _ => return Err("unreviewed native root kind".into()),
    };
    assert_eq!(parent.kind().as_str(), root_kind);
    let expected_roots = ["a", "b"].into_iter().map(|suffix| resource_key(&account, &region, root_kind, &parent_id(family, suffix)))
        .collect::<TestResult<BTreeSet<_>>>()?;
    assert_eq!(observations.iter().map(|row| row.key().clone()).collect::<BTreeSet<_>>(), expected_roots);
    let mut expected_resources = expected_roots.clone();
    let mut expected_edges = BTreeSet::new();
    for suffix in ["a", "b"] {
        let owner = parent_id(family, suffix);
        for case in family.children {
            for resumed in [false, true] {
                let native = child_body(family, case, &owner, resumed)?;
                for key in page_keys(case, &owner, &native, &account, &region)? {
                    expected_edges.insert((key.kind.to_string(), key.id.to_string(), root_kind.to_owned(), owner.clone(), relation.to_owned()));
                    assert!(expected_resources.insert(key), "duplicate native fixture identity");
                }
            }
        }
    }
    assert_eq!(children.len(), family.children.len().checked_mul(2).ok_or("child count overflow")?);
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("native-children.sqlite");
    let mut store = BinaryStore::create(&path, SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?).await?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, root_request.into_bytes())?).await?;
    let lease = store.claim(&sweep, 0, 10_000).await?.ok_or("parent lease absent")?;
    store.commit_page(PageCommit { lease, observations, children, unresolved: Vec::new(), edges: Vec::new(), tags: Vec::new(), next_token: None, continuation_digest: None, done: true, observed_at_ms: 0 }).await?;
    let mut seen = expected_roots;
    // When: durable child requests and their continuation pages are encoded, decoded, projected and committed.
    for at in 1..=32 {
        let Some(lease) = store.claim(&sweep, at, 10_000).await? else { break; };
        let plan = catalog.lookup(family.service, lease.scope().operation().as_str()).ok_or("child plan absent")?;
        let case = family.children.iter().find(|case| case.operation == plan.operation_name()).ok_or("unexpected child scheduled")?;
        assert_eq!(plan.kind().as_str(), case.kind);
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let parent_value = text(&base, &compiled.fields, family.input_key)?;
        assert!([parent_id(family,"a"), parent_id(family,"b")].contains(&parent_value));
        assert_eq!(lease.scope().parent().ok_or("child lost durable parent")?.id.as_str(), parent_value);
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        let resumed = lease.token().is_some();
        if family.service == "batch" {
            let body: Value = serde_json::from_slice(wire.body())?;
            assert_eq!(body.get("jobQueue").and_then(Value::as_str), Some(parent_value.as_str()));
            if case.operation == "ListJobs" {
                assert_eq!(body.get("filters"), Some(&json!([{"name":"JOB_NAME","values":["*"]}])));
                assert_eq!(body.get("jobStatus"), None);
                assert_eq!(body.get("arrayJobId"), None);
                assert_eq!(body.get("multiNodeJobId"), None);
            }
            assert_eq!(body.get("nextToken").and_then(Value::as_str), resumed.then_some("later"));
        } else {
            assert!(wire.uri().path().contains(&format!("/apis/{parent_value}/")) || wire.uri().path().contains(&format!("/restapis/{parent_value}/")));
            let query = wire.uri().query().unwrap_or("");
            assert!(!query.contains("embed"));
            assert_eq!(query.contains("later"), resumed);
        }
        if resumed { assert_eq!(text(&actual, &compiled.fields, if family.token == "position" { "position" } else if family.service == "apigatewayv2" { "NextToken" } else { "nextToken" })?, "later"); }
        let native = child_body(family, case, &parent_value, resumed)?;
        let expected_page = page_keys(case, &parent_value, &native, &account, &region)?;
        let response = decode(plan, &native)?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(&catalog, &context, lease.scope(), &actual, &response, &ancestry)?;
        assert!(children.is_empty());
        assert_eq!(observations.len(), expected_page.len());
        assert_eq!(observations.iter().map(|row| row.key().clone()).collect::<BTreeSet<_>>(), expected_page);
        for observation in &observations { assert!(seen.insert(observation.key().clone()), "duplicate projected native key"); }
        let unresolved = child::policy_dfgk_tests::projected_references(&catalog, &context, lease.scope(), &actual, &response, &observations)?;
        assert_eq!(unresolved.len(), observations.len());
        assert!(unresolved.iter().all(|reference| reference.target.value().as_str() == parent_value));
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        assert_eq!(next.is_some(), !resumed);
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| (Some(next.cursor.into_bytes()), next.continuation_digest));
        store.commit_page(PageCommit { lease, observations, children, unresolved, edges: Vec::new(), tags: Vec::new(), next_token, continuation_digest, done, observed_at_ms: at }).await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then: both parents retain every distinct canonical child, including later pages and all seven Batch statuses.
    assert!(store.claim(&sweep, 33, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(seen, expected_resources);
    assert_eq!(resources.len(), expected_resources.len());
    assert_eq!(resources.into_iter().map(|resource| resource.key).collect::<BTreeSet<_>>(), expected_resources);
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), expected_edges.len());
    let actual_edges = edges.edges.into_iter().map(|edge| {
        assert_eq!(edge.source, "reviewed-rule");
        assert_eq!(edge.from.account, account);
        assert_eq!(edge.to.account, account);
        assert_eq!(edge.from.region, region);
        assert_eq!(edge.to.region, region);
        (edge.from.kind.to_string(), edge.from.id.to_string(), edge.to.kind.to_string(), edge.to.id.to_string(), edge.relation.to_string())
    }).collect::<BTreeSet<_>>();
    assert_eq!(actual_edges, expected_edges);
    Ok(())
}

#[tokio::test]
async fn given_overlapping_rest_api_children_when_later_pages_resume_then_parent_qualified_resources_and_joins_survive() -> TestResult {
    consume_family(&Family { service:"api-gateway", date:"2015-07-09", root:"GetRestApis", root_items:"item", root_key:"id", input_key:"restApiId", token:"position", children:&[
        ChildCase { operation:"GetAuthorizers", kind:"AWS::ApiGateway::Authorizer", items:"item", id:"id" },
        ChildCase { operation:"GetRequestValidators", kind:"AWS::ApiGateway::RequestValidator", items:"item", id:"id" },
        ChildCase { operation:"GetResources", kind:"AWS::ApiGateway::Resource", items:"item", id:"id" },
    ] }).await
}

#[tokio::test]
async fn given_overlapping_v2_api_children_when_later_pages_resume_then_routes_and_authorizers_keep_their_api() -> TestResult {
    consume_family(&Family { service:"apigatewayv2", date:"2018-11-29", root:"GetApis", root_items:"items", root_key:"apiId", input_key:"ApiId", token:"nextToken", children:&[
        ChildCase { operation:"GetRoutes", kind:"AWS::ApiGatewayV2::Route", items:"items", id:"routeId" },
        ChildCase { operation:"GetAuthorizers", kind:"AWS::ApiGatewayV2::Authorizer", items:"items", id:"authorizerId" },
    ] }).await
}

#[tokio::test]
async fn given_two_queues_when_structured_all_name_filter_runs_then_every_status_and_later_quota_share_is_durable() -> TestResult {
    consume_family(&Family { service:"batch", date:"2016-08-10", root:"DescribeJobQueues", root_items:"jobQueues", root_key:"jobQueueArn", input_key:"jobQueue", token:"nextToken", children:&[
        ChildCase { operation:"ListJobs", kind:"AWS::ApiOnly::BatchJob", items:"jobSummaryList", id:"jobId" },
        ChildCase { operation:"ListQuotaShares", kind:"AWS::Batch::QuotaShare", items:"quotaShares", id:"quotaShareArn" },
    ] }).await
}


#[tokio::test]
async fn given_custom_audit_resources_when_paginated_then_exact_custom_enums_and_canonical_rows_are_persisted() -> TestResult {
    filtered_roots("auditmanager", "2017-07-25", &[
        ("ListAssessmentFrameworks","frameworkMetadataList","arn","frameworkType","Custom","AWS::AuditManager::AssessmentFramework"),
        ("ListControls","controlMetadataList","arn","controlType","Custom","AWS::ApiOnly::AuditManagerControl"),
    ]).await
}

#[tokio::test]
async fn given_account_owned_automation_resources_when_paginated_then_serialized_account_enum_is_compiled_and_sent() -> TestResult {
    filtered_roots("bedrock-data-automation", "2023-07-26", &[
        ("ListBlueprints","blueprints","blueprintArn","resourceOwner","ACCOUNT","AWS::Bedrock::Blueprint"),
        ("ListDataAutomationProjects","projects","projectArn","resourceOwner","ACCOUNT","AWS::Bedrock::DataAutomationProject"),
    ]).await
}
