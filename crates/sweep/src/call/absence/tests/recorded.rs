use super::*;
use inventory_engine::limiter::ApiError;

mod captured_messages;
mod code_only;

const MACIE_ABSENT: &[u8] =
    br#"{"__type":"AccessDeniedException","message":"Macie is not enabled."}"#;
const ACCOUNT_ABSENT: &[u8] = br#"{"__type":"ResourceNotFoundException","message":"No GovCloud account is linked to account 000000000000"}"#;
const AMBIGUOUS: &[u8] = br#"{"__type":"AmbiguousService","message":"2 emulated services all claim requests for 'timestream': timestream, timestream; the request is rejected rather than served by an arbitrary one"}"#;
const INDEX: &[u8] = br#"{"Arn":"arn:aws:resource-explorer-2:us-east-1:000000000000:index/9f24251f-92fa-8528-a378-2f20bcb3a177","Type":"LOCAL","State":"CREATING","CreatedAt":1795574719.628248084,"LastUpdatedAt":1795574719.628248084,"Tags":{}}"#;

fn wire(status: u16, content_type: &'static str, body: &'static [u8]) -> TestResult<Response> {
    Ok(Response {
        status: http::StatusCode::from_u16(status)?,
        headers: http::HeaderMap::from_iter([(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static(content_type),
        )]),
        body: bytes::Bytes::from_static(body),
        version: http::Version::HTTP_11,
    })
}

fn assert_absence(plan: &CallablePlan<'_>, response: Response, reason: &str) -> TestResult {
    let answer = validate_status(plan, response)?.decode(plan, false);
    match answer {
        Err(SweepError::Absent(actual)) => assert_eq!(actual, reason),
        _ => return Err("provider absence was not recognized".into()),
    }
    Ok(())
}

#[test]
fn given_matching_fms_absence_message_when_status_is_429_then_call_is_throttled() -> TestResult {
    let compiled = fixture::compile("fms", "2018-01-01", &["ListPolicies"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("fms", "ListPolicies").ok_or("plan absent")?;
    let response = wire(429, "application/cbor", captured_messages::FMS)?;
    assert!(matches!(
        validate_status(plan, response),
        Err(ApiError::Throttled(reason))
            if reason == "AWS service failure: AccessDenied (AccessDeniedException)"
    ));
    Ok(())
}

#[test]
fn given_govcloud_not_found_when_decoded_then_only_the_requested_accounts_missing_link_is_absent()
-> TestResult {
    let operation = "GetGovCloudAccountInformation";
    let compiled = fixture::compile("account", "2021-02-01", &[operation])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("account", operation).ok_or("plan absent")?;
    assert_absence(
        plan,
        wire(404, "application/json", ACCOUNT_ABSENT)?,
        "AWS service failure: Fatal (ResourceNotFoundException)",
    )?;
    let not_found = wire(
        404,
        "application/json",
        br#"{"__type":"ResourceNotFoundException","message":"Account 000000000000 does not exist"}"#,
    )?;
    let other_account = crate::call::validate_status(
        plan,
        wire(404, "application/json", ACCOUNT_ABSENT)?,
        "111111111111",
        REGION,
    );
    for failure in [validate_status(plan, not_found), other_account] {
        assert!(matches!(
            failure,
            Err(ApiError::Permanent(reason))
                if reason == "AWS service failure: Fatal (ResourceNotFoundException)"
        ));
    }
    Ok(())
}

#[test]
fn given_macie_denial_when_decoded_then_only_unconfigured_message_is_absent() -> TestResult {
    let compiled = fixture::compile("macie2", "2020-01-01", &["ListMembers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("macie2", "ListMembers")
        .ok_or("plan absent")?;
    let mut absent = wire(403, "application/json", MACIE_ABSENT)?;
    absent.headers.insert(
        "x-amzn-errortype",
        http::HeaderValue::from_static("AccessDeniedException"),
    );
    assert_absence(
        plan,
        absent,
        "AWS service failure: AccessDenied (AccessDeniedException)",
    )?;
    let denied = wire(
        403,
        "application/json",
        br#"{"__type":"AccessDeniedException","message":"not authorized to perform macie2:ListMembers","details":{"message":"Macie is not enabled."}}"#,
    )?;
    assert!(matches!(
        validate_status(plan, denied),
        Err(ApiError::Denied(reason))
            if reason == "AWS service failure: AccessDenied (AccessDeniedException)"
    ));
    Ok(())
}

#[test]
fn given_macie_header_and_conflicting_body_code_when_decoded_then_scope_remains_denied()
-> TestResult {
    let compiled = fixture::compile("macie2", "2020-01-01", &["ListMembers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("macie2", "ListMembers")
        .ok_or("plan absent")?;
    let mut response = wire(
        403,
        "application/json",
        br#"{"__type":"Other","message":"Macie is not enabled."}"#,
    )?;
    response.headers.insert(
        "x-amzn-errortype",
        http::HeaderValue::from_static("AccessDeniedException"),
    );
    assert!(matches!(
        validate_status(plan, response),
        Err(ApiError::Denied(reason))
            if reason == "AWS service failure: AccessDenied (AccessDeniedException)"
    ));
    Ok(())
}

#[test]
fn given_duplicate_macie_error_headers_when_they_conflict_then_scope_remains_denied() -> TestResult
{
    let compiled = fixture::compile("macie2", "2020-01-01", &["ListMembers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("macie2", "ListMembers")
        .ok_or("plan absent")?;
    let mut response = wire(
        403,
        "application/json",
        br#"{"message":"Macie is not enabled."}"#,
    )?;
    for value in ["AccessDeniedException", "ResourceNotFoundException"] {
        response
            .headers
            .append("x-amzn-errortype", http::HeaderValue::from_static(value));
    }
    assert!(matches!(
        validate_status(plan, response),
        Err(ApiError::Denied(reason))
            if reason == "HTTP 403 Forbidden: invalid response: invalid service error code"
    ));
    Ok(())
}

#[test]
fn given_timestream_routing_ambiguity_when_decoded_then_only_that_server_code_is_permanent()
-> TestResult {
    let service = "timestream-query";
    let operation = "DescribeEndpoints";
    let bytes = crate::execution::test_catalog::compile(
        service,
        "2018-11-01",
        &[operation, "ListScheduledQueries"],
        None,
        |_, _| Ok(()),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
    let content_type = "application/x-amz-json-1.0";
    assert!(matches!(
        validate_status(plan, wire(500, content_type, AMBIGUOUS)?),
        Err(ApiError::Permanent(reason)) if reason == "AWS service failure: Fatal (AmbiguousService)"
    ));
    let unknown = wire(500, content_type, br#"{"__type":"NewServerFailure"}"#)?;
    assert!(matches!(
        validate_status(plan, unknown),
        Err(ApiError::Transient(reason)) if reason == "AWS service failure: Retryable (NewServerFailure)"
    ));
    Ok(())
}

#[test]
fn given_numeric_get_index_dates_when_decoded_then_modeled_date_time_remains_strict() -> TestResult
{
    let service = "resource-explorer-2";
    let compiled = fixture::compile(service, "2022-07-28", &["GetIndex"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup(service, "GetIndex").ok_or("plan absent")?;
    let response = wire(200, "application/json", INDEX)?;
    let result = validate_status(plan, response)?.decode(plan, false);
    assert!(matches!(
        result,
        Err(SweepError::Decode(
            aws_wire::response::DecodeError::Invalid("invalid timestamp")
        ))
    ));
    Ok(())
}

fn get_index_not_found(body: &'static [u8]) -> TestResult<Response> {
    let mut response = wire(404, "application/json", body)?;
    response.headers.insert(
        "x-amzn-errortype",
        http::HeaderValue::from_static("ResourceNotFoundException"),
    );
    Ok(response)
}

#[test]
fn given_get_index_not_found_header_when_body_denies_access_then_scope_fails() -> TestResult {
    let service = "resource-explorer-2";
    let compiled = fixture::compile(service, "2022-07-28", &["GetIndex"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup(service, "GetIndex").ok_or("plan absent")?;
    let response = get_index_not_found(br#"{"__type":"AccessDeniedException"}"#)?;
    assert!(matches!(
        validate_status(plan, response),
        Err(ApiError::Permanent(reason))
            if reason == "AWS service failure: Fatal (ResourceNotFoundException)"
    ));
    Ok(())
}

#[test]
fn given_get_index_not_found_header_when_body_code_agrees_then_scope_is_absent() -> TestResult {
    let service = "resource-explorer-2";
    let compiled = fixture::compile(service, "2022-07-28", &["GetIndex"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup(service, "GetIndex").ok_or("plan absent")?;
    let response = get_index_not_found(
        br#"{"__type":"com.amazonaws.resourceexplorer2#ResourceNotFoundException"}"#,
    )?;
    assert_absence(
        plan,
        response,
        "AWS service failure: Fatal (ResourceNotFoundException)",
    )
}

#[test]
fn given_get_index_not_found_header_when_body_has_no_readable_code_then_scope_is_absent()
-> TestResult {
    let service = "resource-explorer-2";
    let compiled = fixture::compile(service, "2022-07-28", &["GetIndex"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup(service, "GetIndex").ok_or("plan absent")?;
    let bodies: &[&[u8]] = &[
        b"",
        b"not JSON",
        br#"{"detail":{"code":"AccessDeniedException"}}"#,
    ];
    for &body in bodies {
        assert_absence(
            plan,
            get_index_not_found(body)?,
            "AWS service failure: Fatal (ResourceNotFoundException)",
        )?;
    }
    Ok(())
}
