#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
#[path = "../src/response/json/fixture_oracle.rs"]
mod fixture_oracle;
#[path = "protocol_fixtures/load.rs"]
mod protocol_fixtures;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeError, DecodeLimits, ResponseContext},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use http::{HeaderMap, StatusCode};
use inventory_core::catalog::Catalog;
use protocol_fixtures::load;
use serde_json::Value as Json;

type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn published_rpcv2_cbor_success_responses_decode_to_exact_records() -> TestResult {
    let source = load("rpcv2Cbor.json")?;
    let fixture: Json = serde_json::from_str(&source)?;
    let shapes = field(&fixture, "shapes")?
        .as_object()
        .ok_or("fixture shapes")?;
    let mut service_ids: Vec<String> = Vec::new();
    for (id, entry) in shapes {
        if entry["type"] == "service"
            && entry["traits"].get("smithy.protocols#rpcv2Cbor").is_some()
            && has_response_cases(shapes, entry)?
        {
            service_ids.push(id.clone());
        }
    }
    let mut client_cases = 0usize;
    let mut server_cases = 0usize;
    let mut bounds_tested = false;
    for service_id in service_ids {
        let bytes = fixture_catalog::compile_service(&fixture, &service_id)?;
        let catalog = Catalog::borrow(&bytes)?;
        let service_name = service_id.rsplit('#').next().ok_or("service name")?;
        let service = catalog
            .service(&service_name.replace('.', "-"))?
            .ok_or("fixture service")?;
        let service_shape = shape(shapes, &service_id)?;
        let operation_refs = field(service_shape, "operations")?
            .as_array()
            .ok_or("operations")?;
        for operation_ref in operation_refs {
            let operation_id = field(operation_ref, "target")?
                .as_str()
                .ok_or("operation target")?;
            let operation_name = operation_id.rsplit('#').next().ok_or("operation name")?;
            let mut found = None;
            for entry in service.operations()? {
                let item = entry?;
                if item.name()? == operation_name {
                    found = Some(item);
                    break;
                }
            }
            let operation = found.ok_or("catalog operation")?;
            let cases = http_response_cases(shape(shapes, operation_id)?);
            for case in cases {
                if case["appliesTo"] == "server" {
                    server_cases += 1;
                    continue;
                }
                let code = case["code"].as_u64().unwrap_or(200);
                if !(200..300).contains(&code) {
                    continue;
                }
                let id = case["id"].as_str().ok_or("fixture case id")?;
                let body = match case["body"].as_str() {
                    Some(encoded) => STANDARD
                        .decode(encoded)
                        .map_err(|error| format!("{id}: {error}"))?,
                    None => Vec::new(),
                };
                let status = StatusCode::from_u16(u16::try_from(code)?)?;
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
                    let body_limits = DecodeLimits::new(1, 65_536, 32)?;
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            body: &unknown_nested,
                            limits: body_limits,
                            ..context
                        }),
                        Err(DecodeError::Limit("response body"))
                    ));
                    let node_limits = DecodeLimits::new(16_777_216, 2, 32)?;
                    assert!(matches!(
                        response::cbor::decode(ResponseContext {
                            body: &unknown_nested,
                            limits: node_limits,
                            ..context
                        }),
                        Err(DecodeError::Limit("response nodes"))
                    ));
                    let depth_limits = DecodeLimits::new(16_777_216, 65_536, 1)?;
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
                .map_err(|error| format!("{id}: {error}"))?;
                fixture_oracle::assert_fixture_root(
                    &record,
                    service,
                    operation.output()?,
                    &case["params"],
                    id,
                );
                client_cases += 1;
            }
        }
    }
    assert_eq!(client_cases, 40, "published client success response cases");
    assert_eq!(server_cases, 2, "server-only cases are tracked separately");
    Ok(())
}

fn field<'a>(value: &'a Json, key: &str) -> Result<&'a Json, Box<dyn std::error::Error>> {
    value
        .get(key)
        .ok_or_else(|| format!("fixture field `{key}` missing").into())
}

fn shape<'a>(
    shapes: &'a serde_json::Map<String, Json>,
    id: &str,
) -> Result<&'a Json, Box<dyn std::error::Error>> {
    shapes
        .get(id)
        .ok_or_else(|| format!("fixture shape `{id}` missing").into())
}

fn http_response_cases(shape: &Json) -> impl Iterator<Item = &Json> {
    shape
        .get("traits")
        .and_then(|traits| traits.get("smithy.test#httpResponseTests"))
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
}

fn has_response_cases(
    shapes: &serde_json::Map<String, Json>,
    service: &Json,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Some(operations) = service.get("operations").and_then(Json::as_array) else {
        return Ok(false);
    };
    for operation in operations {
        let target = field(operation, "target")?
            .as_str()
            .ok_or("operation target")?;
        if http_response_cases(shape(shapes, target)?).next().is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}
