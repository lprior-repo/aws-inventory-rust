use super::{
    ChildProbeOutcome, ChildProbeSamples, ProbeChildren, ProbeScope, first_page_fields, input,
    probe_children, record, routing,
};
use crate::{
    credentials::TestCredentials, execution::bootstrap::account_id, execution::test_catalog,
    request, test_bindings as fixture,
};
use aws_wire::transport::{Client, Config};
use aws_wire::{
    request::Protocol,
    response::{DecodeError, DecodeLimits, ResponseContext, decode},
};
use catalog_tool::overlay::{
    Binding, ContextSource, ExecutionBranch, FixedValue, MemberPath, OperationName, OverlayClass,
    Scope,
};
use http::{HeaderMap, Method, StatusCode};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose, RuntimeCatalog},
    inventory::{AccountId, Region, RegionId},
    record::{Record, Time, Value},
};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use std::collections::BTreeMap;

#[path = "../../../harness/tests/probe/server.rs"]
mod server;

const ACCOUNT: &str = "111122223333";
const BROWSERS: &str = r#"{
    "browserSummaries": [{
        "browserId": "testBrowser-abcdefghij",
        "browserArn": "arn:aws:bedrock-agentcore:us-east-1:111122223333:browser/testBrowser-abcdefghij",
        "status": "READY",
        "createdAt": "2026-11-23T15:06:46.337384223Z",
        "lastUpdatedAt": "2026-11-23T15:06:47Z"
    }]
}"#;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn given_discovered_account_when_probing_list_analyses_then_uri_contains_caller_account()
-> TestResult {
    let account = caller_account()?;
    let bytes = compile("quicksight", "2018-04-01", "ListAnalyses")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("quicksight", "ListAnalyses")
        .ok_or("ListAnalyses plan absent")?;
    let region = RegionId::parse("us-east-1")?;
    let scope = Region::Regional(region.clone());
    let branch = plan.branches().first();
    let endpoint = routing::resolve(plan, &region)?;

    let base = input::root_bindings_branch(plan, branch, Some(&account), Some(&scope))?;
    let wire = request::encode(plan, &endpoint.url, &base, &first_page_fields(plan))?;

    assert_eq!(account.as_str(), ACCOUNT);
    assert_eq!(wire.method(), Method::GET);
    assert_eq!(wire.uri().path(), "/accounts/111122223333/analyses");
    Ok(())
}

#[test]
fn given_list_browsers_fixture_when_decoding_then_only_modeled_date_time_is_accepted() -> TestResult
{
    let service = "bedrock-agentcore-control";
    let bytes = compile(service, "2023-06-05", "ListBrowsers")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(service, "ListBrowsers")
        .ok_or("ListBrowsers plan absent")?;
    let numeric = BROWSERS.replace("\"2026-11-23T15:06:46.337384223Z\"", "1795446406.337384223");

    let valid = decode_fixture(plan, Protocol::RestJson1, BROWSERS.as_bytes())?;
    let rows = record::item_records(&valid, plan.items())?;
    let invalid = decode_fixture(plan, Protocol::RestJson1, numeric.as_bytes());

    assert_eq!(rows.len(), 1);
    let row = rows.first().ok_or("browser summary absent")?;
    let fields = catalog_tool::smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let created_at = fields.id("createdAt").ok_or("createdAt field absent")?;
    let timestamp = row
        .root()
        .get_field(created_at)?
        .ok_or("timestamp absent")?;
    assert_eq!(
        timestamp.to_value()?,
        Value::Time(Time::new(1_795_446_406, 337_384_223)?)
    );
    assert!(matches!(
        invalid,
        Err(DecodeError::Invalid("invalid timestamp"))
    ));
    Ok(())
}

fn compile(service: &str, version: &str, operation: &str) -> TestResult<Vec<u8>> {
    test_catalog::compile(service, version, &[operation], None, |_, _| Ok(()))
}

fn caller_account() -> TestResult<AccountId> {
    let bytes = compile("sts", "2011-06-15", "GetCallerIdentity")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("sts", "GetCallerIdentity")
        .ok_or("identity plan absent")?;
    let PlanPurpose::AccountIdentity { account_path } = plan.purpose() else {
        return Err("identity plan purpose absent".into());
    };
    let body = format!(
        "<GetCallerIdentityResponse><GetCallerIdentityResult><Account>{ACCOUNT}</Account><Arn>arn:aws:iam::{ACCOUNT}:user/test</Arn><UserId>test</UserId></GetCallerIdentityResult></GetCallerIdentityResponse>"
    );
    let response = decode_fixture(plan, Protocol::AwsQuery, body.as_bytes())?;
    Ok(account_id(&response, account_path)?)
}

fn decode_fixture(
    plan: &CallablePlan<'_>,
    protocol: Protocol,
    body: &[u8],
) -> Result<Record, DecodeError> {
    decode(ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body,
        headers: &HeaderMap::new(),
        status: StatusCode::OK,
        limits: DecodeLimits::default().excluding_members(plan.excluded_members()),
    })
}

#[tokio::test]
async fn given_regional_or_global_parent_when_probing_regional_child_then_signing_region_follows_scope()
-> TestResult {
    for (scope, expected) in [(Scope::Region, "us-west-2"), (Scope::Account, "us-east-1")] {
        let regions = MemberPath::new("Regions")?;
        let bytes = fixture::compiled_with_parents(
            1,
            |parent| {
                parent.scope = Some(scope);
                parent.bind.remove(&regions);
            },
            |_| {},
        )?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let (outcomes, requests) = probe_fixture(
            &catalog,
            "us-west-2",
            ChildProbeSamples::default(),
            &["ListParents0"],
            server::response,
        )
        .await?;
        assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
        let request = requests.last().ok_or("child not sent")?;
        assert_eq!(
            server::header(request, "authorization")?
                .split("Credential=")
                .nth(1)
                .and_then(|scope| scope.split('/').nth(2)),
            Some(expected)
        );
        assert!(
            matches!(outcomes.as_slice(), [("ListChildren", ChildProbeOutcome::Called { items: 1, errors })] if errors.is_empty())
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_error_in_first_branch_when_probing_child_then_later_bindable_branch_is_sent()
-> TestResult {
    let organization = MemberPath::new("OrganizationId")?;
    let first = branch("bad", BTreeMap::from([(organization, Binding::Account)]));
    let entity = MemberPath::new("EntityId")?;
    let mut second = branch("good", BTreeMap::new());
    second
        .fixed
        .insert(entity, FixedValue::String("chosen-branch".into()));
    let bytes = fixture::compiled(1, |child| child.branches.extend([first, second]))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (outcomes, requests) = probe_fixture(
        &catalog,
        "us-east-1",
        ChildProbeSamples::default(),
        &["ListParents0"],
        server::response,
    )
    .await?;
    assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
    assert_eq!(
        body(requests.last().ok_or("child not sent")?)?.get("EntityId"),
        Some(&serde_json::json!("chosen-branch"))
    );
    assert!(
        matches!(outcomes.as_slice(), [("ListChildren", ChildProbeOutcome::Called { items: 1, errors })] if errors.is_empty())
    );
    Ok(())
}

#[tokio::test]
async fn given_second_producer_only_has_items_when_probing_alternatives_then_child_uses_its_values()
-> TestResult {
    let bytes = fixture::compiled(2, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (outcomes, requests) = probe_fixture(
        &catalog,
        "us-east-1",
        ChildProbeSamples::default(),
        &["ListParents0", "ListParents1"],
        |request| match server::target(request)? {
            "ListParents0" => Ok((200, r#"{"Items":[]}"#)),
            "ListParents1" => Ok((200, r#"{"Items":[{"Id":"from-second"}]}"#)),
            _ => server::response(request),
        },
    )
    .await?;
    assert_eq!(
        targets(&requests)?,
        ["ListParents0", "ListParents1", "ListChildren"]
    );
    assert_eq!(
        body(requests.last().ok_or("child not sent")?)?,
        serde_json::json!({"OrganizationId":"org-1", "EntityId":"from-second", "Regions":["us-east-1"]})
    );
    assert!(
        matches!(outcomes.as_slice(), [("ListChildren", ChildProbeOutcome::Called { items: 1, errors })] if errors.is_empty())
    );
    Ok(())
}

#[tokio::test]
async fn given_staged_operation_cycle_when_probing_children_then_reentry_terminates() -> TestResult
{
    let bytes = staged_cycle()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let samples = ChildProbeSamples::new(64).ok_or("invalid test limit")?;
    let (mut outcomes, requests) = probe_fixture(
        &catalog,
        "us-east-1",
        samples,
        &["ListParents0"],
        |request| match server::target(request)? {
            "ListParents0" => Ok((200, r#"{"Items":[{"Id":"root"}]}"#)),
            "ListParents1" => Ok((200, r#"{"Items":[{"Id":"intermediate"}]}"#)),
            _ => server::response(request),
        },
    )
    .await?;
    assert_eq!(
        targets(&requests)?,
        [
            "ListParents0",
            "ListChildren",
            "ListParents1",
            "ListChildren"
        ]
    );
    let ids = requests
        .iter()
        .skip(1)
        .map(|request| body(request).map(|body| body.get("EntityId").cloned()))
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        ids,
        ["root", "child", "intermediate"].map(|id| Some(serde_json::json!(id)))
    );
    outcomes.sort_unstable_by_key(|(name, _)| *name);
    assert!(matches!(outcomes.as_slice(), [
        ("ListChildren", ChildProbeOutcome::Called { items: 2, errors: first }),
        ("ListParents1", ChildProbeOutcome::Called { items: 1, errors: second }),
    ] if first.is_empty() && second.is_empty()));
    Ok(())
}

fn staged_cycle() -> TestResult<Vec<u8>> {
    let root = OperationName::new("ListParents0")?;
    let b = OperationName::new("ListParents1")?;
    let entity = MemberPath::new("EntityId")?;
    let id = MemberPath::selected("Id")?;
    let organization = MemberPath::new("OrganizationId")?;
    let b_bindings = seed_bindings()?;
    let detail = BTreeMap::from([(
        entity.clone(),
        Binding::From {
            operation: b,
            path: id.clone(),
        },
    )]);
    fixture::compiled_with_parents(
        2,
        |plan| {
            if plan.fixed.get(&organization) == Some(&FixedValue::String("org-1".into())) {
                plan.class = Some(OverlayClass::Expand);
                plan.fixed.clear();
                plan.bind.extend(b_bindings.clone());
            }
        },
        |plan| {
            plan.bind.insert(
                entity,
                Binding::From {
                    operation: root.clone(),
                    path: id,
                },
            );
            plan.bind.insert(
                organization.clone(),
                Binding::Context {
                    operation: root,
                    source: ContextSource::Request,
                    path: organization.clone(),
                },
            );
            plan.branches.extend([
                branch("a-seed", BTreeMap::new()),
                branch("b-detail", detail),
            ]);
        },
    )
}

fn seed_bindings() -> TestResult<[(MemberPath, Binding); 2]> {
    let operation = OperationName::new("ListChildren")?;
    let organization = MemberPath::new("OrganizationId")?;
    Ok([
        (
            MemberPath::new("EntityId")?,
            Binding::FromBranch {
                operation: operation.clone(),
                branch: "a-seed".into(),
                path: MemberPath::selected("Id")?,
            },
        ),
        (
            organization.clone(),
            Binding::ContextFromBranch {
                operation,
                branch: "a-seed".into(),
                source: ContextSource::Request,
                path: organization,
            },
        ),
    ])
}

fn branch(discriminator: &str, bind: BTreeMap<MemberPath, Binding>) -> ExecutionBranch {
    ExecutionBranch {
        discriminator: discriminator.into(),
        scope: Scope::Region,
        request_region: None,
        bind,
        fixed: BTreeMap::new(),
        emission: None,
        row_policy: None,
    }
}

async fn probe_fixture<'a>(
    catalog: &'a RuntimeCatalog<'_>,
    root_region: &str,
    samples: ChildProbeSamples,
    roots: &[&str],
    respond: impl Fn(&str) -> TestResult<(u16, &'static str)>,
) -> TestResult<(Vec<(&'a str, ChildProbeOutcome)>, Vec<String>)> {
    let server = server::Server::new().await?;
    let endpoint = server.endpoint()?;
    let root_region = RegionId::parse(root_region)?;
    let home = RegionId::parse("us-east-1")?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..Config::default()
    })?;
    let credentials = TestCredentials::new("us-east-1");
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let scope = ProbeScope {
        region: &home,
        account: None,
        endpoint_override: Some(&endpoint),
    };
    let plans: Vec<_> = catalog.plans().iter().collect();
    let probe = async {
        let mut parents = Vec::new();
        for operation in roots {
            let plan = catalog
                .lookup("bindings", operation)
                .ok_or("root plan missing")?;
            let answer = super::call_once(
                catalog,
                &credentials,
                &client,
                &executor,
                plan,
                ProbeScope {
                    region: &root_region,
                    ..scope
                },
            )
            .await?;
            parents.push((plan, answer));
        }
        let outcomes = probe_children(
            catalog,
            &credentials,
            &client,
            &executor,
            scope,
            ProbeChildren::new(&plans, parents, samples),
        )
        .await?;
        fixture_outcomes(catalog, outcomes)
    };
    server.capture_with(probe, respond).await
}

fn fixture_outcomes<'a>(
    catalog: &'a RuntimeCatalog<'_>,
    outcomes: Vec<(&CallablePlan<'_>, ChildProbeOutcome)>,
) -> TestResult<Vec<(&'a str, ChildProbeOutcome)>> {
    outcomes
        .into_iter()
        .map(|(plan, outcome)| {
            let operation = catalog
                .lookup(plan.service_name(), plan.operation_name())
                .ok_or("reported child missing from catalog")?
                .operation_name();
            Ok((operation, outcome))
        })
        .collect()
}

fn targets(requests: &[String]) -> TestResult<Vec<&str>> {
    requests
        .iter()
        .map(|request| server::target(request))
        .collect()
}

fn body(request: &str) -> TestResult<serde_json::Value> {
    let (_, body) = request.split_once("\r\n\r\n").ok_or("body missing")?;
    Ok(serde_json::from_str(body)?)
}
