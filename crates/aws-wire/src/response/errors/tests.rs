use super::decode;
use super::{ErrorClass, ServiceError};
use crate::request::Protocol;
use crate::response::{DecodeError, DecodeLimits, ResponseContext};
use catalog_tool::smithy::{Registry, build_catalog};
use inventory_core::catalog::{Catalog, Target};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};

mod limits;
mod modeled;
mod published;
mod support;
use support::*;

#[test]
fn json_error_code_is_normalized_without_retaining_message() {
    let parsed = super::json::code(
        br#"{"__type":"example#ThrottlingException:httpStatusCode=429","message":"private"}"#,
        DecodeLimits::default(),
    );
    assert_eq!(
        parsed.ok().flatten().as_deref(),
        Some("ThrottlingException")
    );
}

#[test]
fn xml_error_code_is_extracted_from_query_envelope() {
    let parsed = super::xml::code(
        b"<ErrorResponse><Error><Code>AccessDenied</Code><Message>private</Message></Error></ErrorResponse>",
        DecodeLimits::default(),
    );
    assert_eq!(parsed.ok().flatten().as_deref(), Some("AccessDenied"));
}

#[test]
fn cbor_error_code_is_extracted_from_rpc_envelope() {
    let parsed = super::cbor::code(
        &[
            0xa2, 0x66, b'_', b'_', b't', b'y', b'p', b'e', 0x6c, b'A', b'c', b'c', b'e', b's',
            b's', b'D', b'e', b'n', b'i', b'e', b'd', 0x67, b'm', b'e', b's', b's', b'a', b'g',
            b'e', 0x66, b's', b'e', b'c', b'r', b'e', b't',
        ],
        DecodeLimits::default(),
    );
    assert_eq!(parsed.ok().flatten().as_deref(), Some("AccessDenied"));
}

#[test]
fn malformed_envelopes_are_not_hidden_by_status_fallback() {
    let limits = DecodeLimits::default();
    assert!(super::json::code(b"{not-json", limits).is_err());
    assert!(super::xml::code(b"<Error><Code>oops</Error>", limits).is_err());
    assert!(super::cbor::code(&[0xa1, 0x66, b'_', b'_', b't', b'y', b'p', b'e'], limits).is_err());
}

#[test]
fn every_public_class_has_a_deterministic_rule() {
    assert_eq!(
        super::class_from_code(Some("ThrottlingException")),
        Some(ErrorClass::Throttle)
    );
    assert_eq!(
        super::class_from_status(http::StatusCode::SERVICE_UNAVAILABLE),
        ErrorClass::Retryable
    );
    assert_eq!(
        super::class_from_code(Some("AccessDenied")),
        Some(ErrorClass::AccessDenied)
    );
    assert_eq!(
        super::class_from_status(http::StatusCode::NOT_IMPLEMENTED),
        ErrorClass::NotSupported
    );
    assert_eq!(
        super::class_from_code(Some("ExpiredToken")),
        Some(ErrorClass::ExpiredToken)
    );
    assert_eq!(
        super::class_from_code(Some("ExpiredSignature")),
        Some(ErrorClass::ExpiredSignature)
    );
    assert_eq!(super::class_from_code(Some("SignatureDoesNotMatch")), None);
    assert_eq!(
        super::class_from_code(Some("InvalidSignatureException")),
        None
    );
    assert_eq!(
        super::class_from_status(http::StatusCode::BAD_REQUEST),
        ErrorClass::Fatal
    );
}

#[test]
fn service_error_display_includes_code_for_diagnosis() {
    let error = ServiceError {
        class: ErrorClass::AccessDenied,
        code: Some("SecretAccessDenied".to_owned()),
        retry_after: None,
    };
    assert_eq!(
        error.to_string(),
        "AWS service failure: AccessDenied (SecretAccessDenied)"
    );
    let no_code = ServiceError {
        class: ErrorClass::AccessDenied,
        code: None,
        retry_after: None,
    };
    assert_eq!(no_code.to_string(), "AWS service failure: AccessDenied");
}

#[test]
fn code_normalization_rejects_oversized_or_control_values() {
    assert_eq!(
        super::normalize_code("ns#ExpiredToken:status=403")
            .ok()
            .as_deref(),
        Some("ExpiredToken")
    );
    assert!(super::normalize_code(&"x".repeat(257)).is_err());
    assert!(super::normalize_code("bad\ncode").is_err());
}

mod malformed;

mod alternate_envelope;
