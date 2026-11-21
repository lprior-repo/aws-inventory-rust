#[path = "response_xml_fixtures/behavior.rs"]
mod behavior;
#[path = "response_xml_fixtures/bounds.rs"]
mod bounds;
use bounds::{limits_are_enforced, malformed_xml_is_rejected};
#[path = "../src/response/json/fixture_oracle.rs"]
mod fixture_oracle;
#[path = "response_xml_fixtures/mod.rs"]
mod response_xml_fixtures;

use aws_wire::{
    request::Protocol,
    response::{DecodeLimits, ResponseContext},
};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use inventory_core::catalog::Catalog;
use serde_json::Value as Json;
use std::error::Error;

use response_xml_fixtures::model;

struct Suite {
    name: &'static str,
    protocol_id: &'static str,
    protocol: Protocol,
    fixture: &'static str,
    successful_clients: usize,
    servers: usize,
    error_lane: usize,
}

#[test]
fn published_aws_query_success_responses_match_params() -> Result<(), Box<dyn Error>> {
    run(Suite {
        name: "awsQuery",
        protocol_id: "aws.protocols#awsQuery",
        protocol: Protocol::AwsQuery,
        fixture: include_str!("fixtures/protocol-tests/awsQuery.json"),
        successful_clients: 36,
        servers: 0,
        error_lane: 3,
    })
}

#[test]
fn published_ec2_query_success_responses_match_params() -> Result<(), Box<dyn Error>> {
    run(Suite {
        name: "ec2Query",
        protocol_id: "aws.protocols#ec2Query",
        protocol: Protocol::Ec2Query,
        fixture: include_str!("fixtures/protocol-tests/ec2Query.json"),
        successful_clients: 27,
        servers: 0,
        error_lane: 2,
    })
}

#[test]
fn published_rest_xml_success_responses_match_params() -> Result<(), Box<dyn Error>> {
    run(Suite {
        name: "restXml",
        protocol_id: "aws.protocols#restXml",
        protocol: Protocol::RestXml,
        fixture: include_str!("fixtures/protocol-tests/restXml.json"),
        successful_clients: 81,
        servers: 2,
        error_lane: 3,
    })
}

fn run(suite: Suite) -> Result<(), Box<dyn Error>> {
    let fixture: Json = serde_json::from_str(suite.fixture)?;
    let shapes = fixture
        .get("shapes")
        .and_then(Json::as_object)
        .ok_or("fixture shapes")?;
    let service_ids = shapes
        .iter()
        .filter(|(_, shape)| {
            shape["type"] == "service" && shape["traits"].get(suite.protocol_id).is_some()
        })
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let mut successful: usize = 0;
    let mut servers: usize = 0;
    let mut error_lane = error_case_count(shapes);
    for service_id in service_ids {
        let catalog_bytes = model::compile_service(&fixture, &service_id)?;
        let catalog = Catalog::borrow(&catalog_bytes)?;
        let service_name = service_id.rsplit('#').next().ok_or("service name")?;
        let service = catalog
            .service(&service_name.replace('.', "-"))?
            .ok_or("compiled service exists")?;
        for operation_id in service_operations(shapes, &service_id)? {
            let operation_name = operation_id.rsplit('#').next().ok_or("operation name")?;
            let mut found = None;
            for entry in service.operations()? {
                let entry = entry?;
                if entry.name()? == operation_name {
                    found = Some(entry);
                    break;
                }
            }
            let operation = found.ok_or("compiled catalog operation")?;
            for case in response_cases(shapes, &operation_id) {
                if case["appliesTo"] == "server" {
                    servers = servers.saturating_add(1);
                } else if is_success(case) {
                    decode_case(&suite, service, operation, case, successful == 0)?;
                    successful = successful.saturating_add(1);
                } else {
                    error_lane = error_lane.saturating_add(1);
                }
            }
        }
    }
    assert_eq!(
        successful, suite.successful_clients,
        "{} successful client cases",
        suite.name
    );
    assert_eq!(servers, suite.servers, "{} server cases", suite.name);
    assert_eq!(
        error_lane, suite.error_lane,
        "{} error-lane cases",
        suite.name
    );
    Ok(())
}

fn service_operations(
    shapes: &serde_json::Map<String, Json>,
    service_id: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let service = shapes.get(service_id).ok_or("service shape")?;
    let mut names = Vec::new();
    for item in service["operations"]
        .as_array()
        .ok_or("service operations")?
    {
        names.push(
            item["target"]
                .as_str()
                .ok_or("operation target")?
                .to_owned(),
        );
    }
    Ok(names)
}

fn response_cases<'a>(shapes: &'a serde_json::Map<String, Json>, id: &str) -> &'a [Json] {
    shapes
        .get(id)
        .and_then(|shape| shape.get("traits"))
        .and_then(|traits| traits.get("smithy.test#httpResponseTests"))
        .and_then(Json::as_array)
        .map_or(&[], Vec::as_slice)
}

fn error_case_count(shapes: &serde_json::Map<String, Json>) -> usize {
    shapes
        .values()
        .filter(|shape| {
            shape
                .get("traits")
                .and_then(|traits| traits.get("smithy.api#error"))
                .is_some_and(Json::is_string)
        })
        .map(|shape| {
            shape
                .get("traits")
                .and_then(|traits| traits.get("smithy.test#httpResponseTests"))
                .and_then(Json::as_array)
                .map_or(0, Vec::len)
        })
        .sum()
}

fn is_success(case: &Json) -> bool {
    case["code"].as_u64().unwrap_or(200) < 300
}

fn decode_case(
    suite: &Suite,
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
    case: &Json,
    test_invalid: bool,
) -> Result<(), Box<dyn Error>> {
    let headers = headers(case)?;
    let context = context(suite, service, operation, case, &headers);
    let id = case["id"].as_str().ok_or("response case ID")?;
    let record = aws_wire::response::decode(context)
        .map_err(|error| format!("{} {id}: {error}", suite.name))?;
    fixture_oracle::assert_fixture_root(
        &record,
        service,
        operation.output()?,
        &case["params"],
        &format!("{} {id}", suite.name),
    );
    if suite.name == "restXml" && id == "XmlEmptyLists" {
        missing_lists_remain_absent(context, service, operation, &record)?;
    }
    if test_invalid {
        malformed_xml_is_rejected(context);
        limits_are_enforced(context)?;
    }
    Ok(())
}

fn missing_lists_remain_absent(
    context: ResponseContext<'_>,
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
    populated: &inventory_core::record::Record,
) -> Result<(), Box<dyn Error>> {
    let xml = std::str::from_utf8(context.body)?;
    let without_lists = xml
        .replace("<stringList/>", "")
        .replace("<stringSet></stringSet>", "");
    let absent = aws_wire::response::decode(ResponseContext {
        body: without_lists.as_bytes(),
        ..context
    })?;
    fixture_oracle::assert_fixture_root(
        &absent,
        service,
        operation.output()?,
        &serde_json::json!({}),
        "XmlEmptyLists omitted members",
    );
    assert_ne!(
        populated, &absent,
        "present empty lists differ from omitted lists"
    );
    Ok(())
}

fn headers(case: &Json) -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    if let Some(values) = case["headers"].as_object() {
        for (name, value) in values {
            let header_name = HeaderName::from_bytes(name.as_bytes())?;
            let header_value =
                HeaderValue::from_str(value.as_str().ok_or("fixture header string")?)?;
            headers.append(header_name, header_value);
        }
    }
    Ok(headers)
}

fn context<'a>(
    suite: &Suite,
    service: inventory_core::catalog::Service<'a>,
    operation: inventory_core::catalog::Operation<'a>,
    case: &'a Json,
    headers: &'a HeaderMap,
) -> ResponseContext<'a> {
    let body = case["body"].as_str().unwrap_or("").as_bytes();
    let status = case["code"]
        .as_u64()
        .and_then(|code| u16::try_from(code).ok())
        .and_then(|code| StatusCode::from_u16(code).ok())
        .unwrap_or(StatusCode::OK);
    ResponseContext {
        service,
        operation,
        protocol: suite.protocol,
        body,
        headers,
        status,
        limits: DecodeLimits::default(),
    }
}
