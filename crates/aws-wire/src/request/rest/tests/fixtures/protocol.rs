use super::{CompressionPolicy, Protocol, RequestContext, RequestLimits, input, model};
use crate::request::{EncodeError, rest};
use inventory_core::catalog::Catalog;
use serde_json::json;

#[test]
fn rest_requests_reject_protocols_absent_from_service_metadata() {
    for (protocol_id, selected, wrong) in [
        (
            "aws.protocols#restJson1",
            Protocol::RestJson1,
            Protocol::RestXml,
        ),
        (
            "aws.protocols#restXml",
            Protocol::RestXml,
            Protocol::RestJson1,
        ),
    ] {
        let fixture = json!({"shapes": {
            "test#Boundary": {
                "type": "service", "version": "2026-11-14",
                "traits": {"aws.api#service": {"sdkId": "Boundary"}, (protocol_id): {}},
                "operations": [{"target": "test#Probe"}]
            },
            "test#Probe": {
                "type": "operation", "input": {"target": "test#Input"},
                "traits": {"smithy.api#http": {"method": "POST", "uri": "/probe", "code": 200}}
            },
            "test#Input": {
                "type": "structure", "members": {"Value": {"target": "smithy.api#String"}}
            }
        }});
        let bytes = model::compile_service(&fixture, "test#Boundary");
        let catalog = Catalog::borrow(&bytes).expect("valid regression catalog");
        let service = catalog
            .service("Boundary")
            .expect("service lookup")
            .expect("service");
        let operation = service
            .operations()
            .expect("operations")
            .next()
            .expect("operation")
            .expect("valid operation");
        let record = input::record(service, operation, &json!({}));
        let context = RequestContext {
            service,
            operation,
            protocol: selected,
            input: record.root(),
            endpoint: "https://example.test",
            limits: RequestLimits::default(),
            idempotency_token: None,
            compression: CompressionPolicy::Disabled,
            addressing: None,
        };
        let request = rest::encode(context).expect("declared protocol encodes");
        assert_eq!(request.method(), http::Method::POST);
        assert_eq!(request.uri().to_string(), "https://example.test/probe");
        assert!(
            matches!(
                rest::encode(RequestContext {
                    protocol: wrong,
                    ..context
                }),
                Err(EncodeError::Unsupported(
                    "protocol does not match service metadata"
                ))
            ),
            "{protocol_id} must reject {wrong:?}"
        );
    }
}
