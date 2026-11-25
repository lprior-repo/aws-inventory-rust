use super::{cursor, request_fields};
use crate::{consumer_fixture, input, request};
use inventory_core::catalog::runtime::{CallablePlan, RuntimeCatalog};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn first_request(plan: &CallablePlan<'_>) -> TestResult<http::Request<bytes::Bytes>> {
    let base = input::root_bindings(plan, None, None)?;
    let fields = request_fields(&cursor::Cursor::default(), plan, None)?;
    Ok(request::encode(
        plan,
        "https://example.test",
        &base,
        &fields,
    )?)
}

#[test]
fn given_imagebuilder_lists_when_requests_are_encoded_then_all_eleven_ask_for_25_items()
-> TestResult {
    let cases = [
        ("ListComponentBuildVersions", json!({"maxResults": 25})),
        ("ListComponents", json!({"maxResults": 25, "owner": "Self"})),
        (
            "ListContainerRecipes",
            json!({"maxResults": 25, "owner": "Self"}),
        ),
        ("ListDistributionConfigurations", json!({"maxResults": 25})),
        ("ListImagePipelines", json!({"maxResults": 25})),
        (
            "ListImageRecipes",
            json!({"maxResults": 25, "owner": "Self"}),
        ),
        ("ListImages", json!({"maxResults": 25, "owner": "Self"})),
        (
            "ListInfrastructureConfigurations",
            json!({"maxResults": 25}),
        ),
        ("ListLifecyclePolicies", json!({"maxResults": 25})),
        ("ListWorkflowBuildVersions", json!({"maxResults": 25})),
        ("ListWorkflows", json!({"maxResults": 25, "owner": "Self"})),
    ];
    let operations = cases.each_ref().map(|(operation, _)| *operation);
    let compiled = consumer_fixture::compile("imagebuilder", "2019-12-02", &operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for (operation, expected) in cases {
        let plan = catalog.lookup("imagebuilder", operation).ok_or(operation)?;
        let request = first_request(plan)?;
        assert_eq!(
            serde_json::from_slice::<Value>(request.body())?,
            expected,
            "{operation}"
        );
    }
    Ok(())
}

#[test]
fn given_chime_voice_phone_numbers_when_requested_then_query_uses_the_99_item_limit() -> TestResult
{
    let compiled =
        consumer_fixture::compile("chime-sdk-voice", "2022-08-03", &["ListPhoneNumbers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("chime-sdk-voice", "ListPhoneNumbers")
        .ok_or("phone number plan missing")?;
    assert_eq!(
        first_request(plan)?
            .uri()
            .path_and_query()
            .map(http::uri::PathAndQuery::as_str),
        Some("/phone-numbers?max-results=99")
    );
    Ok(())
}

#[test]
fn given_mediapackage_origin_endpoints_when_requested_then_query_uses_50_items_and_no_channel_filter()
-> TestResult {
    let compiled =
        consumer_fixture::compile("mediapackage", "2017-10-12", &["ListOriginEndpoints"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("mediapackage", "ListOriginEndpoints")
        .ok_or("origin endpoint plan missing")?;
    assert_eq!(
        first_request(plan)?
            .uri()
            .path_and_query()
            .map(http::uri::PathAndQuery::as_str),
        Some("/origin_endpoints?maxResults=50")
    );
    Ok(())
}
