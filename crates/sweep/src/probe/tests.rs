use super::{first_page_fields, input, record, routing};
use crate::{execution::bootstrap::account_id, execution::test_catalog, request};
use aws_wire::{
    request::Protocol,
    response::{DecodeError, DecodeLimits, ResponseContext, decode},
};
use http::{HeaderMap, Method, StatusCode};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose, RuntimeCatalog},
    inventory::{AccountId, Region, RegionId},
    record::{Record, Time, Value},
};

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
    // Given the account returned by the modeled identity call.
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

    // When the root's first-page bindings are encoded with that account.
    let base = input::root_bindings_branch(plan, branch, Some(&account), Some(&scope))?;
    let wire = request::encode(plan, &endpoint.url, &base, &first_page_fields(plan))?;

    // Then the modeled REST label is bound to the caller, not a dummy account.
    assert_eq!(account.as_str(), ACCOUNT);
    assert_eq!(wire.method(), Method::GET);
    assert_eq!(wire.uri().path(), "/accounts/111122223333/analyses");
    Ok(())
}

#[test]
fn given_list_browsers_fixture_when_decoding_then_only_modeled_date_time_is_accepted() -> TestResult
{
    // Given a compiled ListBrowsers plan and two JSON representations of its timestamp.
    let service = "bedrock-agentcore-control";
    let bytes = compile(service, "2023-06-05", "ListBrowsers")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(service, "ListBrowsers")
        .ok_or("ListBrowsers plan absent")?;
    let numeric = BROWSERS.replace("\"2026-11-23T15:06:46.337384223Z\"", "1795446406.337384223");

    // When the real wire decoder reads each response and the collector selects items.
    let valid = decode_fixture(plan, Protocol::RestJson1, BROWSERS.as_bytes())?;
    let rows = record::item_records(&valid, plan.items())?;
    let invalid = decode_fixture(plan, Protocol::RestJson1, numeric.as_bytes());

    // Then the modeled format preserves nanoseconds and the nonconforming one fails.
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
