mod iam;
mod recorded;
use super::*;
use crate::consumer_fixture as fixture;
use inventory_core::catalog::runtime::RuntimeCatalog;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const ACCOUNT: &str = "000000000000";
const REGION: &str = "us-east-1";

fn validate_status(
    plan: &CallablePlan<'_>,
    response: Response,
) -> Result<ValidatedResponse, inventory_engine::limiter::ApiError> {
    super::super::validate_status(plan, response, ACCOUNT, REGION)
}

const ABSENT: &[(&str, &str, &str)] = &[
    ("iam", "GetLoginProfile", "NoSuchEntity"),
    (
        "organizations",
        "ListAccounts",
        "AWSOrganizationsNotInUseException",
    ),
    (
        "organizations",
        "DescribeResourcePolicy",
        "ResourcePolicyNotFoundException",
    ),
    (
        "organizations",
        "ListDelegatedServicesForAccount",
        "AccountNotRegisteredException",
    ),
    (
        "organizations",
        "DescribeEffectivePolicy",
        "EffectivePolicyNotFoundException",
    ),
    (
        "mgn",
        "DescribeSourceServers",
        "UninitializedAccountException",
    ),
    (
        "drs",
        "DescribeSourceServers",
        "UninitializedAccountException",
    ),
    (
        "compute-optimizer",
        "DescribeRecommendationExportJobs",
        "OptInRequiredException",
    ),
    (
        "compute-optimizer-automation",
        "ListAutomationRules",
        "OptInRequiredException",
    ),
    (
        "ec2",
        "DescribeSpotDatafeedSubscription",
        "InvalidSpotDatafeed.NotFound",
    ),
    (
        "resource-explorer-2",
        "GetIndex",
        "ResourceNotFoundException",
    ),
    (
        "s3-control",
        "GetAccessGrantsInstance",
        "AccessGrantsInstanceNotExistsError",
    ),
    (
        "s3-control",
        "ListAccessGrants",
        "AccessGrantsInstanceNotExistsError",
    ),
    (
        "s3-control",
        "ListAccessGrantsLocations",
        "AccessGrantsInstanceNotExistsError",
    ),
];
const FAILURES: &[(&str, &str, &str)] = &[
    ("iam", "GetLoginProfile", "AccessDenied"),
    ("iam", "GetLoginProfile", "ServiceFailure"),
    ("iam", "GetLoginProfile", "ResourceNotFoundException"),
    ("iam", "GetUser", "NoSuchEntity"),
    (
        "organizations",
        "DescribeResourcePolicy",
        "AccessDeniedException",
    ),
    ("organizations", "DescribePolicy", "PolicyNotFoundException"),
    (
        "organizations",
        "DescribeAccount",
        "AccountNotFoundException",
    ),
    (
        "resource-explorer-2",
        "GetView",
        "ResourceNotFoundException",
    ),
    (
        "securitylake",
        "ListSubscribers",
        "ResourceNotFoundException",
    ),
    ("securityhub", "DescribeHub", "InvalidAccessException"),
    ("macie2", "ListMembers", "AccessDeniedException"),
    (
        "ec2",
        "DescribeTrunkInterfaceAssociations",
        "OperationNotPermitted",
    ),
    (
        "s3-control",
        "GetAccessGrant",
        "AccessGrantsInstanceNotExistsError",
    ),
    (
        "controltower",
        "ListEnabledControls",
        "ResourceNotFoundException",
    ),
];

#[test]
fn given_absent_features_when_classified_then_specific_codes_are_empty() {
    for &(service, operation, code) in ABSENT {
        assert!(known(service, operation, code), "{service} {operation}");
    }
}

#[test]
fn given_missing_resources_or_access_denied_when_classified_then_failures_remain() {
    for &(service, operation, code) in FAILURES {
        assert!(!known(service, operation, code), "{service} {operation}");
    }
}

fn response(code: &str) -> Response {
    Response {
        status: http::StatusCode::BAD_REQUEST,
        headers: http::HeaderMap::from_iter([(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/x-amz-json-1.1"),
        )]),
        body: bytes::Bytes::from(format!(r#"{{"__type":"{code}"}}"#)),
        version: http::Version::HTTP_11,
    }
}

#[test]
fn given_organizations_not_in_use_when_wire_call_finishes_then_absence_retains_reason() -> TestResult
{
    let compiled = fixture::compile("organizations", "2016-11-28", &["ListAccounts"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("organizations", "ListAccounts")
        .ok_or("plan absent")?;
    let answer = validate_status(plan, response("AWSOrganizationsNotInUseException"))?.decode(plan, false);
    match answer {
        Err(SweepError::Absent(reason)) => {
            assert!(reason.contains("AWSOrganizationsNotInUseException"))
        }
        _ => return Err("absence was not retained".into()),
    }
    assert!(matches!(
        validate_status(plan, response("AccessDeniedException")),
        Err(inventory_engine::limiter::ApiError::Denied(_))
    ));
    Ok(())
}
