#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
#[path = "../src/response/json/fixture_oracle.rs"]
mod fixture_oracle;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeError, DecodeLimits, ResponseContext},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use http::{HeaderMap, StatusCode};
use inventory_core::catalog::Catalog;
use serde_json::Value as Json;

#[test]
fn published_rpcv2_cbor_success_responses_decode_to_exact_records() {
    let fixture: Json =
        serde_json::from_str(include_str!("fixtures/protocol-tests/rpcv2Cbor.json"))
            .expect("valid CBOR fixture");
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let service_ids: Vec<String> = shapes
        .iter()
        .filter_map(|(id, shape)| {
            (shape["type"] == "service"
                && shape["traits"].get("smithy.protocols#rpcv2Cbor").is_some()
                && has_response_cases(shapes, shape))
            .then(|| id.clone())
        })
        .collect();
    let mut client_cases = 0usize;
    let mut server_cases = 0usize;
    let mut bounds_tested = false;
    for service_id in service_ids {
        let bytes = fixture_catalog::compile_service(&fixture, &service_id)?;
        let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
        let service_name = service_id.rsplit('#').next().expect("service name");
        let service = catalog
            .service(&service_name.replace('.', "-"))
            .expect("service lookup")
            .expect("fixture service");
        for operation_ref in shapes[&service_id]["operations"]
            .as_array()
            .expect("operations")
        {
            let operation_id = operation_ref["target"].as_str().expect("operation target");
            let operation_name = operation_id.rsplit('#').next().expect("operation name");
            let operation = service
                .operations()
                .expect("operation iterator")
                .map(|entry| entry.expect("valid operation"))
                .find(|item| item.name().expect("operation name") == operation_name)
                .expect("catalog operation");
            let cases = shapes[operation_id]["traits"]["smithy.test#httpResponseTests"]
                .as_array()
                .into_iter()
                .flatten();
            for case in cases {
                if case["appliesTo"] == "server" {
                    server_cases += 1;
                    continue;
                }
                let code = case["code"].as_u64().unwrap_or(200);
                if !(200..300).contains(&code) {
                    continue;
                }
                let id = case["id"].as_str().expect("fixture case id");
                let body = match case["body"].as_str() {
                    Some(encoded) => STANDARD
                        .decode(encoded)
                        .unwrap_or_else(|error| panic!("{id}: {error}")),
                    None => Vec::new(),
                };
                let status = StatusCode::from_u16(u16::try_from(code).expect("status code"))
                    .expect("valid status");
                let headers = HeaderMap::new();
                let context = ResponseContext {
                    service,
                    operation,
                    protocol: Protocol::RpcV2Cbor,
                    body: &body,
                    headers: &headers,
                    status,
                    limits: DecodeLimits::default(),
                };
                if !bounds_tested {
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            status: StatusCode::BAD_REQUEST,
                            ..context
                        }),
                        Err(DecodeError::Invalid(
                            "non-success response belongs to error decoder"
                        ))
                    ));
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            protocol: Protocol::AwsJson11,
                            ..context
                        }),
                        Err(DecodeError::Unsupported("CBOR response protocol"))
                    ));
                    assert!(
                        response::cbor::decode(ResponseContext {
                            body: &[0xa1],
                            ..context
                        })
                        .is_err()
                    );
                    assert!(
                        response::cbor::decode(ResponseContext {
                            body: &[0xa0, 0x00],
                            ..context
                        })
                        .is_err()
                    );
                    let unknown_nested = [0xa1, 0x61, b'?', 0xa1, 0x61, b'x', 0x80];
                    let body_limits = DecodeLimits::new(1, 65_536, 32).expect("valid body limits");
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            body: &unknown_nested,
                            limits: body_limits,
                            ..context
                        }),
                        Err(DecodeError::Limit("response body"))
                    ));
                    let node_limits =
                        DecodeLimits::new(16_777_216, 2, 32).expect("valid node limits");
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            body: &unknown_nested,
                            limits: node_limits,
                            ..context
                        }),
                        Err(DecodeError::Limit("response nodes"))
                    ));
                    let depth_limits =
                        DecodeLimits::new(16_777_216, 65_536, 1).expect("valid depth limits");
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            body: &unknown_nested,
                            limits: depth_limits,
                            ..context
                        }),
                        Err(DecodeError::Limit("response depth"))
                    ));
                    bounds_tested = true;
                }
                let record = response::decode(ResponseContext {
                    service,
                    operation,
                    protocol: Protocol::RpcV2Cbor,
                    body: &body,
                    headers: &headers,
                    status,
                    limits: DecodeLimits::default(),
                })
                .unwrap_or_else(|error| panic!("{id}: {error}"));
                fixture_oracle::assert_fixture_root(
                    &record,
                    service,
                    operation.output().expect("output shape"),
                    &case["params"],
                    id,
                );
                client_cases += 1;
            }
        }
    }
    assert_eq!(client_cases, 40, "published client success response cases");
    assert_eq!(server_cases, 2, "server-only cases are tracked separately");
}

fn has_response_cases(shapes: &serde_json::Map<String, Json>, service: &Json) -> bool {
    service["operations"].as_array().is_some_and(|operations| {
        operations.iter().any(|operation| {
            let target = operation["target"].as_str().expect("operation target");
            shapes[target]["traits"]["smithy.test#httpResponseTests"]
                .as_array()
                .is_some_and(|cases| !cases.is_empty())
        })
    })
}
