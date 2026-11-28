use super::*;

const CASES: &[(&str, &str, &str, &str, &str, http::StatusCode)] = &[
    (
        "redshift-serverless",
        "2021-04-21",
        "ListTableRestoreStatus",
        "ResourceNotFoundException",
        "Workgroup and namespace not found. Set up resources with the create-namespace and create-workgroups operations.",
        http::StatusCode::NOT_FOUND,
    ),
    (
        "sesv2",
        "2019-09-27",
        "ListImportJobs",
        "BadRequestException",
        "Your account is still in the sandbox.",
        http::StatusCode::BAD_REQUEST,
    ),
];

#[test]
fn given_scoped_not_enabled_messages_when_classified_then_outcome_is_not_applicable() -> TestResult {
    for &(service, version, operation, code, message, status) in CASES {
        let compiled = fixture::compile(service, version, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
        let result = validate_status(plan, response(status, code, message)?)?.decode(plan, false);
        assert!(
            matches!(result, Err(SweepError::NotEnabled(_))),
            "{service}:{operation} was not classified as not applicable"
        );
    }
    Ok(())
}

#[test]
fn given_near_miss_or_non_bad_request_when_classified_then_it_is_not_not_applicable() -> TestResult {
    for &(service, version, operation, code, message, applicable_status) in CASES {
        let compiled = fixture::compile(service, version, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
        assert!(
            !matches!(
                validate_status(plan, response(
                    applicable_status,
                    code,
                    &format!("{message} extra")
                )?),
                Ok(ValidatedResponse::NotEnabled(_))
            ),
            "{service}:{operation} accepted a near-match message"
        );
        for status in [
            http::StatusCode::BAD_REQUEST,
            http::StatusCode::NOT_FOUND,
            http::StatusCode::FORBIDDEN,
            http::StatusCode::TOO_MANY_REQUESTS,
            http::StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            if status == applicable_status {
                continue;
            }
            assert!(
                !matches!(
                    validate_status(plan, response(status, code, message)?),
                    Ok(ValidatedResponse::NotEnabled(_) | ValidatedResponse::Absent(_))
                ),
                "{service}:{operation}:{status} was classified as empty"
            );
        }
    }
    Ok(())
}

#[test]
fn given_matching_message_for_another_operation_when_classified_then_it_is_not_not_applicable()
-> TestResult {
    for (service, version, operation, status) in [
        (
            "redshift-serverless",
            "2021-04-21",
            "ListWorkgroups",
            http::StatusCode::NOT_FOUND,
        ),
        (
            "sesv2",
            "2019-09-27",
            "ListMultiRegionEndpoints",
            http::StatusCode::BAD_REQUEST,
        ),
    ] {
        let compiled = fixture::compile(service, version, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
        let original = CASES
            .iter()
            .find(|case| case.0 == service)
            .ok_or("case absent")?;
        assert!(
            !matches!(
                validate_status(plan, response(status, original.3, original.4)?),
                Ok(ValidatedResponse::NotEnabled(_))
            ),
            "{service}:{operation} accepted another operation's prerequisite message"
        );
    }
    Ok(())
}

#[test]
fn given_conflicting_error_code_headers_when_classified_then_prerequisite_is_not_trusted()
-> TestResult {
    let (service, version, operation) = ("sesv2", "2019-09-27", "ListImportJobs");
    let message = "Your account is still in the sandbox.";
    let compiled = fixture::compile(service, version, &[operation])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
    let conflicting = [
        ("x-amzn-errortype", "BadRequestException"),
        ("x-amzn-query-error", "AccessDeniedException"),
    ];
    for pairs in [conflicting, [conflicting[1], conflicting[0]]] {
        let response = response_with_headers(http::StatusCode::BAD_REQUEST, message, &pairs)?;
        assert!(
            !matches!(
                validate_status(plan, response),
                Ok(ValidatedResponse::NotEnabled(_) | ValidatedResponse::Absent(_))
            ),
            "{service}:{operation} trusted prerequisite under conflicting headers {pairs:?}"
        );
    }
    let agreeing = [
        ("x-amzn-errortype", "BadRequestException"),
        ("x-amzn-query-error", "BadRequestException"),
    ];
    for pairs in [agreeing, [agreeing[1], agreeing[0]]] {
        let response = response_with_headers(http::StatusCode::BAD_REQUEST, message, &pairs)?;
        assert!(
            matches!(
                validate_status(plan, response),
                Ok(ValidatedResponse::NotEnabled(_))
            ),
            "{service}:{operation} rejected agreeing headers {pairs:?}"
        );
    }
    Ok(())
}

#[test]
fn given_sitewise_unsupported_feature_message_when_classified_then_it_remains_a_failure()
-> TestResult {
    let compiled = fixture::compile("iotsitewise", "2019-12-02", &["ListWorkspaces"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iotsitewise", "ListWorkspaces")
        .ok_or("plan absent")?;
    assert!(matches!(
        validate_status(
            plan,
            response(
                http::StatusCode::BAD_REQUEST,
                "InvalidRequestException",
                "This feature is not supported yet",
            )?,
        ),
        Err(ApiError::Permanent(_))
    ));
    Ok(())
}

fn response(status: http::StatusCode, code: &str, message: &str) -> TestResult<Response> {
    let body = serde_json::json!({"__type": code, "message": message}).to_string();
    Ok(Response {
        status,
        headers: http::HeaderMap::from_iter([(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        )]),
        body: bytes::Bytes::from(body),
        version: http::Version::HTTP_11,
    })
}

fn response_with_headers(
    status: http::StatusCode,
    message: &str,
    headers: &[(&'static str, &'static str)],
) -> TestResult<Response> {
    let mut map = http::HeaderMap::new();
    map.insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );
    for &(name, value) in headers {
        map.append(name, http::HeaderValue::from_static(value));
    }
    let body = serde_json::json!({"message": message}).to_string();
    Ok(Response {
        status,
        headers: map,
        body: bytes::Bytes::from(body),
        version: http::Version::HTTP_11,
    })
}
