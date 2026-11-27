use super::{cursor, request_fields};
use crate::{consumer_fixture, input, request};
use inventory_core::catalog::runtime::{CallablePlan, RuntimeCatalog};
use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn first_request(plan: &CallablePlan<'_>) -> TestResult<http::Request<bytes::Bytes>> {
    request_with_cursor(plan, &cursor::Cursor::default())
}
fn request_with_cursor(
    plan: &CallablePlan<'_>,
    cursor: &cursor::Cursor,
) -> TestResult<http::Request<bytes::Bytes>> {
    let base = input::root_bindings(plan, None, None)?;
    let fields = request_fields(cursor, plan, None)?;
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

fn assert_ec2_cap_and_token(plan: &CallablePlan<'_>, cap: u32) -> TestResult {
    let first = request_with_cursor(plan, &cursor::Cursor::default())?;
    let first_body = String::from_utf8_lossy(first.body());
    assert!(first_body.contains(&format!("MaxResults={cap}")));
    let token = plan
        .pagination()
        .tokens
        .first()
        .ok_or("EC2 page token binding missing")?;
    let token_field = token
        .response_path
        .first()
        .copied()
        .ok_or("EC2 page token response path missing")?;
    let response = inventory_core::record::Record::encode(
        &inventory_core::record::Value::Map(std::collections::BTreeMap::from([(
            inventory_core::record::Key::Field(token_field),
            inventory_core::record::Value::Text("next-cursor".to_owned()),
        )])),
    )?;
    let next = cursor::Cursor::default()
        .next(
            plan.pagination(),
            &response,
            &http::HeaderMap::new(),
            10,
        )?
        .ok_or("expected EC2 continuation")?;
    let resumed = cursor::Cursor::decode(Some(next.cursor.as_bytes()))?;
    let continuation = request_with_cursor(plan, &resumed)?;
    let continuation_body = String::from_utf8_lossy(continuation.body());
    assert!(continuation_body.contains(&format!("MaxResults={cap}")));
    assert!(continuation_body.contains("NextToken=next-cursor"));
    Ok(())
}

#[test]
fn given_ec2_page_caps_when_requests_are_prepared_then_caps_and_next_tokens_reach_wire()
-> TestResult {
    let cases = [
        ("DescribeAddressTransfers", 10),
        ("DescribeCoipPools", 255),
        ("DescribeIpamPools", 100),
        ("DescribeIpamPrefixListResolverTargets", 100),
        ("DescribeIpamPrefixListResolvers", 100),
        ("DescribeIpamResourceDiscoveries", 100),
        ("DescribeIpamResourceDiscoveryAssociations", 100),
        ("DescribeIpamScopes", 100),
        ("DescribeIpams", 100),
        ("DescribeIpv6Pools", 10),
        ("DescribeVerifiedAccessEndpoints", 200),
        ("DescribeVerifiedAccessGroups", 200),
    ];
    let operations = cases.each_ref().map(|(operation, _)| *operation);
    let compiled = crate::execution::test_catalog::compile(
        "ec2",
        "2016-11-15",
        &operations,
        None,
        |_, _| Ok(()),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    for (operation, cap) in cases {
        let plan = catalog.lookup("ec2", operation).ok_or(operation)?;
        assert_ec2_cap_and_token(plan, cap)?;
    }
    Ok(())
}
