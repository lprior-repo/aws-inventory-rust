use super::{fixture_oracle, model};
use aws_wire::{
    request::Protocol,
    response::{self, DecodeError, DecodeLimits, ResponseContext},
};
use http::{HeaderMap, StatusCode};
use inventory_core::catalog::Catalog;
use serde_json::json;

#[test]
fn xml_union_has_exactly_one_known_variant_and_missing_required_scalars_are_corrected() {
    let fixture = json!({"shapes": {
        "example#XmlBoundaries": {"type":"service","version":"1","operations":[{"target":"example#Read"}],"traits":{"aws.protocols#restXml":{}}},
        "example#Read": {"type":"operation","output":{"target":"example#Output"}},
        "example#Output": {"type":"structure","members": {
            "choice":{"target":"example#Choice"},
            "count":{"target":"smithy.api#Integer","traits":{"smithy.api#required":{}}}
        }},
        "example#Choice": {"type":"union","members": {
            "left":{"target":"smithy.api#String"},
            "right":{"target":"smithy.api#String"},
            "none":{"target":"smithy.api#Unit"}
        }}
    }});
    let bytes = model::compile_service(&fixture, "example#XmlBoundaries");
    let catalog = Catalog::borrow(&bytes).expect("valid compiled model");
    let service = catalog
        .service("XmlBoundaries")
        .expect("lookup")
        .expect("service");
    let operation = service
        .operations()
        .expect("operations")
        .next()
        .expect("operation")
        .expect("valid operation");
    let headers = HeaderMap::new();
    let context = ResponseContext {
        service,
        operation,
        protocol: Protocol::RestXml,
        body: b"",
        headers: &headers,
        status: StatusCode::OK,
        limits: DecodeLimits::default(),
    };
    for (body, expected) in [
        (
            b"<Output><choice><left>a&amp;b</left></choice></Output>".as_slice(),
            json!({"choice":{"left":"a&b"},"count":0}),
        ),
        (
            b"<Output><choice><none/></choice></Output>".as_slice(),
            json!({"choice":{"none":{}},"count":0}),
        ),
    ] {
        let record =
            response::decode(ResponseContext { body, ..context }).expect("one known variant");
        fixture_oracle::assert_fixture_root(
            &record,
            service,
            operation.output().expect("output"),
            &expected,
            "XML union and required scalar",
        );
    }
    for body in [
        b"<Output><choice/></Output>".as_slice(),
        b"<Output><choice><unknown>x</unknown></choice></Output>".as_slice(),
        b"<Output><choice><left>a</left><right>b</right></choice></Output>".as_slice(),
        b"<Output><choice><left>a</left><left>b</left></choice></Output>".as_slice(),
    ] {
        assert!(
            matches!(
                response::decode(ResponseContext { body, ..context }),
                Err(DecodeError::Invalid(_))
            ),
            "{body:?}"
        );
    }
}
