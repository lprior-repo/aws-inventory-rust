use super::*;
fn detector_id_from_arn(arn: &str) -> TestResult<&str> {
    arn.split_once('/')
        .map(|(_, tail)| tail)
        .ok_or_else(|| "detector ARN has no resource path".into())
}
fn evs_environment_id(parent_arn: &str) -> TestResult<&str> {
    parent_arn
        .split_once('/')
        .map(|(_, tail)| tail)
        .ok_or_else(|| "environment ARN has no resource path".into())
}
fn json_token_field(service: &str) -> &'static str {
    if matches!(service, "fms" | "secrets-manager") {
        "NextToken"
    } else {
        "nextToken"
    }
}
fn json_size_field(service: &str) -> &'static str {
    if matches!(service, "fms" | "secrets-manager") {
        "MaxResults"
    } else {
        "maxResults"
    }
}
pub(super) fn assert_get(
    wire: &http::Request<bytes::Bytes>,
    path: &str,
    service: &str,
    operation: &str,
    second: bool,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::GET,
        "expected GET for {service}.{operation}"
    );
    assert_eq!(
        wire.uri().path(),
        path,
        "GET path for {service}.{operation}"
    );
    assert!(
        wire.body().is_empty(),
        "GET body must be empty for {service}.{operation}"
    );
    assert!(
        wire.headers().get("x-amz-target").is_none(),
        "GET should not have x-amz-target for {service}.{operation}"
    );
    let query = wire.uri().query().unwrap_or_default();
    if (service, operation) == ("fis", "ListExperimentTargetAccountConfigurations") {
        let expected = if second {
            "nextToken=same-next-token"
        } else {
            ""
        };
        assert_eq!(query, expected, "GET query for {service}.{operation}");
    } else {
        let size = page_size(service, operation)?;
        let expected = if second {
            format!("maxResults={size}&nextToken=same-next-token")
        } else {
            format!("maxResults={size}")
        };
        assert_eq!(query, expected, "GET query for {service}.{operation}");
    }
    Ok(())
}
pub(super) fn assert_post(
    wire: &http::Request<bytes::Bytes>,
    service: &str,
    operation: &str,
    second: bool,
    parent_id: Option<&str>,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::POST,
        "expected POST for {service}.{operation}"
    );
    assert_eq!(
        wire.uri().path(),
        "/",
        "POST path must be / for {service}.{operation}"
    );
    assert!(
        wire.uri().query().is_none(),
        "POST query must be None for {service}.{operation}"
    );
    let target_prefix = json_target(service)?;
    let header = wire
        .headers()
        .get("x-amz-target")
        .ok_or("AWS JSON target missing")?
        .to_str()?;
    let expected_target = format!("{}.{}", target_prefix, operation);
    assert_eq!(
        header, &expected_target,
        "AWS JSON target for {service}.{operation}"
    );
    let encoded: Json = serde_json::from_slice(wire.body())?;
    let mut expected = serde_json::Map::new();
    if let Some(field) = json_binding(service, operation) {
        let value = parent_id.ok_or(format!("missing parent for {service}.{operation}"))?;
        if service == "frauddetector" {
            let id = detector_id_from_arn(value)?;
            expected.insert(field.to_owned(), json!(id));
        } else {
            expected.insert(field.to_owned(), json!(value));
        }
    }
    if service == "evs"
        && operation.starts_with("ListEnvironment")
        && operation != "ListEnvironments"
    {
        let value = parent_id.ok_or("missing parent for EVS child operation")?;
        let env_id = evs_environment_id(value)?;
        expected.insert("environmentId".to_owned(), json!(env_id));
    }
    if service == "secrets-manager" {
        expected.insert("IncludePlannedDeletion".to_owned(), json!(true));
    }
    if operation != "GetNotificationChannel" && operation != "DescribeDetector" {
        let size = page_size(service, operation)?;
        expected.insert(json_size_field(service).to_owned(), json!(size));
    }
    if second {
        expected.insert(
            json_token_field(service).to_owned(),
            json!("same-next-token"),
        );
    }
    assert_eq!(
        encoded,
        Json::Object(expected),
        "POST body for {service}.{operation}"
    );
    Ok(())
}
