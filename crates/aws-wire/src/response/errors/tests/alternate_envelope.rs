use super::*;

fn decode_error(
    body: &[u8],
    headers: &http::HeaderMap,
) -> Result<ServiceError, Box<dyn std::error::Error>> {
    let bytes = compile_service_model(&modeled_error_fixture(), "example#ErrorService")?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("ErrorService")?.ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    Ok(decode(ResponseContext {
        service,
        operation,
        protocol: Protocol::RpcV2Cbor,
        body,
        headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::default(),
    })?)
}

#[test]
fn given_cbor_error_when_decoded_then_modeled_class_and_code_survive() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/cbor"),
    )]);
    let body = b"\xa2\x66__type\x78\x18example#RetryableFailure\x67message\x67private";
    let error = decode_error(body, &headers)?;
    assert_eq!(error.class, ErrorClass::Retryable);
    assert_eq!(error.code.as_deref(), Some("RetryableFailure"));
    assert_eq!(
        error.to_string(),
        "AWS service failure: Retryable (RetryableFailure)"
    );
    Ok(())
}

#[test]
fn given_invalid_primary_header_when_cbor_body_has_code_then_header_error_propagates() -> TestResult
{
    for value in [
        http::HeaderValue::from_static(" "),
        http::HeaderValue::from_bytes(b"AccessDeniedException\xff")?,
    ] {
        let headers = http::HeaderMap::from_iter([(
            http::header::HeaderName::from_static("x-amzn-errortype"),
            value,
        )]);
        let error = decode_error(b"\xa1\x66__type\x70RetryableFailure", &headers)
            .err()
            .ok_or("invalid header was ignored")?;
        assert!(matches!(
            error.downcast_ref::<DecodeError>(),
            Some(DecodeError::Invalid("invalid service error code"))
        ));
    }
    Ok(())
}

#[test]
fn given_repeated_error_header_when_values_differ_then_decode_rejects_it() -> TestResult {
    let mut headers = http::HeaderMap::new();
    let denied = http::HeaderValue::from_static("AccessDeniedException");
    headers.append("x-amzn-errortype", denied.clone());
    headers.append("x-amzn-errortype", denied);
    let body = br#"{"message":"Macie is not enabled."}"#;
    assert_eq!(
        decode_error(body, &headers)?.code.as_deref(),
        Some("AccessDeniedException")
    );
    headers.append(
        "x-amzn-errortype",
        http::HeaderValue::from_static("ResourceNotFoundException"),
    );
    let error = decode_error(body, &headers)
        .err()
        .ok_or("conflicting header was ignored")?;
    assert!(matches!(
        error.downcast_ref::<DecodeError>(),
        Some(DecodeError::Invalid("invalid service error code"))
    ));
    Ok(())
}

#[test]
fn given_conflicting_lower_priority_header_when_decoded_then_primary_denial_survives() -> TestResult
{
    let mut headers = http::HeaderMap::new();
    headers.insert(
        "x-amzn-errortype",
        http::HeaderValue::from_static("AccessDeniedException"),
    );
    headers.append("x-amz-error-code", http::HeaderValue::from_static("A"));
    headers.append("x-amz-error-code", http::HeaderValue::from_static("B"));
    let error = decode_error(b"", &headers)?;
    assert_eq!(error.class, ErrorClass::AccessDenied);
    assert_eq!(error.code.as_deref(), Some("AccessDeniedException"));
    Ok(())
}

#[test]
fn given_query_error_header_when_body_and_retry_hint_are_invalid_then_code_survives() -> TestResult
{
    let headers = http::HeaderMap::from_iter([
        (
            http::header::HeaderName::from_static("x-amzn-query-error"),
            http::HeaderValue::from_static("RetryableFailure;Sender"),
        ),
        (
            http::header::RETRY_AFTER,
            http::HeaderValue::from_bytes(b"\xff")?,
        ),
    ]);
    let error = decode_error(b"not CBOR", &headers)?;
    assert_eq!(error.class, ErrorClass::Retryable);
    assert_eq!(error.code.as_deref(), Some("RetryableFailure"));
    assert_eq!(error.retry_after, None);
    Ok(())
}

#[test]
fn given_json_error_without_media_type_when_rpc_error_is_decoded_then_code_survives() -> TestResult
{
    let error = decode_error(
        br#" {"__type":"SerializationException","message":"private"}"#,
        &http::HeaderMap::new(),
    )?;
    assert_eq!(error.class, ErrorClass::Fatal);
    assert_eq!(error.code.as_deref(), Some("SerializationException"));
    Ok(())
}

#[test]
fn given_json_rpc_error_when_decoded_then_code_is_preserved_without_message() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-amz-json-1.1; charset=utf-8"),
    )]);
    let error = decode_error(br#"{"__type":"SerializationException","message":"missing required header `x-amz-target`"}"#, &headers)?;
    assert_eq!(error.code.as_deref(), Some("SerializationException"));
    assert_eq!(error.class, ErrorClass::Fatal);
    assert!(!error.to_string().contains("x-amz-target"));
    Ok(())
}

#[test]
fn given_unsupported_header_when_body_is_other_protocol_then_gap_remains_visible() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::HeaderName::from_static("x-amzn-errortype"),
        http::HeaderValue::from_static("NotImplemented"),
    )]);
    let error = decode_error(br#"{"message":"unimplemented"}"#, &headers)?;
    assert_eq!(error.code.as_deref(), Some("NotImplemented"));
    assert_eq!(error.class, ErrorClass::NotSupported);
    Ok(())
}

#[test]
fn given_unknown_operation_when_classified_then_absence_is_not_a_decoder_failure() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    )]);
    [
        "NotImplemented",
        "UnknownOperationException",
        "UnknownOperationError",
        "UnknownService",
        "InvalidAction",
    ]
    .into_iter()
    .try_for_each(|code| -> TestResult {
        let body = format!(r#"{{"__type":"{code}"}}"#);
        let error = decode_error(body.as_bytes(), &headers)?;
        assert_eq!(error.class, ErrorClass::NotSupported, "{code}");
        Ok(())
    })
}

#[test]
fn given_resource_or_validation_error_when_classified_then_failure_is_not_hidden() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    )]);
    [
        "ResourceNotFoundException",
        "ValidationException",
        "AccessDeniedException",
    ]
    .into_iter()
    .try_for_each(|code| -> TestResult {
        let body = format!(r#"{{"__type":"{code}"}}"#);
        let error = decode_error(body.as_bytes(), &headers)?;
        assert_ne!(error.class, ErrorClass::NotSupported, "{code}");
        Ok(())
    })
}
