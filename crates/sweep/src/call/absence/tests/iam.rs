use super::*;
use inventory_engine::limiter::ApiError;

fn xml_response(code: &str, status: http::StatusCode) -> Response {
    Response {
        status,
        headers: http::HeaderMap::from_iter([(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/xml"),
        )]),
        body: bytes::Bytes::from(format!(
            "<ErrorResponse><Error><Type>Sender</Type><Code>{code}</Code></Error></ErrorResponse>"
        )),
        version: http::Version::HTTP_11,
    }
}

#[test]
fn given_login_profile_when_missing_or_denied_then_only_no_such_entity_is_empty() -> TestResult {
    let compiled = fixture::compile("iam", "2010-05-08", &["ListUsers", "GetLoginProfile"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "GetLoginProfile")
        .ok_or("profile plan missing")?;
    let validate = |code, status| validate_status(plan, xml_response(code, status));
    let absence = validate("NoSuchEntity", http::StatusCode::NOT_FOUND)?.decode(plan, false);
    assert!(matches!(absence, Err(SweepError::Absent(reason)) if reason.contains("NoSuchEntity")));
    assert!(matches!(
        validate("AccessDenied", http::StatusCode::FORBIDDEN),
        Err(ApiError::Denied(_))
    ));
    assert!(validate("ServiceFailure", http::StatusCode::INTERNAL_SERVER_ERROR).is_err());
    Ok(())
}
