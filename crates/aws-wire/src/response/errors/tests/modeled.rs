use super::*;
#[test]
fn public_decode_uses_modeled_retryability_and_expiry_precedence() {
    let fixture = modeled_error_fixture();
    let bytes = compile_service_model(&fixture, "example#ErrorService");
    let catalog = Catalog::borrow(&bytes).expect("compiled model");
    let service = catalog
        .service("ErrorService")
        .expect("service lookup")
        .expect("service");
    let operation = service
        .operations()
        .expect("operations")
        .next()
        .expect("operation reference")
        .expect("valid operation");
    let expected = [
        ("ThrottleError", ErrorClass::Throttle, 400),
        ("RetryableFailure", ErrorClass::Retryable, 400),
        ("ExpiredToken", ErrorClass::ExpiredToken, 403),
        ("ExpiredSignature", ErrorClass::ExpiredSignature, 403),
        ("SignatureDoesNotMatch", ErrorClass::AccessDenied, 403),
        ("InvalidSignatureException", ErrorClass::AccessDenied, 403),
    ];
    let headers = http::HeaderMap::new();
    for (code, class, status) in expected {
        let body = format!(r#"{{"__type":"{code}","message":"private"}}"#);
        let response = ResponseContext {
            service,
            operation,
            protocol: Protocol::AwsJson11,
            body: body.as_bytes(),
            headers: &headers,
            status: http::StatusCode::from_u16(status).expect("valid status"),
            limits: DecodeLimits::default(),
        };
        let decoded = super::decode(response).expect("modeled public error");
        assert_eq!(decoded.code.as_deref(), Some(code));
        assert_eq!(decoded.class, class);
    }
}
