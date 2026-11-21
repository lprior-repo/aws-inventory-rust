use super::*;
#[test]
fn public_decode_enforces_malformed_body_and_resource_limits() -> TestResult {
    let fixture = modeled_error_fixture();
    let bytes = compile_service_model(&fixture, "example#ErrorService")?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("ErrorService")?.ok_or("service")?;
    let operation = service
        .operations()?
        .next()
        .ok_or("operation reference")??;
    let headers = http::HeaderMap::new();
    let malformed = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"{not-json",
        headers: &headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::default(),
    };
    assert!(matches!(
        super::decode(malformed),
        Err(DecodeError::Invalid("service error JSON syntax"))
    ));

    let oversized = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"{}",
        headers: &headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::new(1, 64, 16)?,
    };
    assert!(matches!(
        super::decode(oversized),
        Err(DecodeError::Limit("response body"))
    ));

    let unknown = br#"{"unknown":{"__type":"ExpiredToken","nested":[{"value":1},{"value":2}]}}"#;
    let node_limited = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: unknown,
        headers: &headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::new(1024, 1, 16)?,
    };
    assert!(matches!(
        super::decode(node_limited),
        Err(DecodeError::Limit("response nodes"))
    ));

    let unknown_response = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: unknown,
        headers: &headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::default(),
    };
    let decoded_unknown = super::decode(unknown_response)?;
    assert_eq!(decoded_unknown.code, None);
    assert_eq!(decoded_unknown.class, ErrorClass::Fatal);
    assert!(!decoded_unknown.to_string().contains("unknown"));

    let depth_limited = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: br#"{"unknown":{"nested":1}}"#,
        headers: &headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::new(1024, 64, 1)?,
    };
    assert!(matches!(
        super::decode(depth_limited),
        Err(DecodeError::Limit("response depth"))
    ));

    let success = ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: b"{}",
        headers: &headers,
        status: http::StatusCode::OK,
        limits: DecodeLimits::default(),
    };
    assert!(matches!(
        super::decode(success),
        Err(DecodeError::Invalid(
            "success response is not a service error"
        ))
    ));
    Ok(())
}
