use super::*;
use aws_wire::response::errors::ErrorClass;

const JSON: &str = "application/json";
const CBOR: &str = "application/cbor";
const XML: &str = "text/xml";
const IAM_DENIED: &str = "AccessDeniedException";

struct Family {
    service: &'static str,
    version: &'static str,
    status: u16,
    media_type: &'static str,
    code: &'static str,
    original_class: ErrorClass,
    captures: &'static [(&'static str, &'static [u8])],
}

pub(super) const FMS: &[u8] = b"\xbf\x66__type\x75AccessDeniedException\x67Message\x78\x4cNo default admin could be found for account 000000000000 in Region us-east-1\xff";
const SECURITY_HUB: &[u8] = br#"{"__type":"InvalidAccessException","Message":"Account 000000000000 is not subscribed to Security Hub CSPM in Region us-east-1"}"#;
const FAMILIES: &[Family] = &[
    Family {
        service: "billingconductor",
        version: "2021-07-30",
        status: 403,
        media_type: JSON,
        code: IAM_DENIED,
        original_class: ErrorClass::AccessDenied,
        captures: &[
            ("ListBillingGroups", br#"{"__type":"AccessDeniedException","Message":"Account 000000000000 is not authorized to call ListBillingGroups. Only payer account is authorized"}"#),
            ("ListCustomLineItems", br#"{"__type":"AccessDeniedException","Message":"Account 000000000000 is not authorized to call ListCustomLineItems. Only payer account is authorized"}"#),
            ("ListPricingPlans", br#"{"__type":"AccessDeniedException","Message":"Account 000000000000 is not authorized to call ListPricingPlans. Only payer account is authorized"}"#),
            ("ListPricingRules", br#"{"__type":"AccessDeniedException","Message":"Account 000000000000 is not authorized to call ListPricingRules. Only payer account is authorized"}"#),
        ],
    },
    Family {
        service: "fms",
        version: "2018-01-01",
        status: 400,
        media_type: CBOR,
        code: IAM_DENIED,
        original_class: ErrorClass::AccessDenied,
        captures: &[
            ("GetNotificationChannel", FMS),
            ("ListAppsLists", FMS),
            ("ListPolicies", FMS),
            ("ListProtocolsLists", FMS),
            ("ListResourceSets", FMS),
        ],
    },
    Family {
        service: "macie2",
        version: "2020-01-01",
        status: 403,
        media_type: JSON,
        code: IAM_DENIED,
        original_class: ErrorClass::AccessDenied,
        captures: &[
            ("ListCustomDataIdentifiers", MACIE_ABSENT),
            ("ListMembers", MACIE_ABSENT),
        ],
    },
    Family {
        service: "securityhub",
        version: "2018-10-26",
        status: 401,
        media_type: JSON,
        code: "InvalidAccessException",
        original_class: ErrorClass::AccessDenied,
        captures: &[
            ("DescribeActionTargets", SECURITY_HUB),
            ("DescribeHub", SECURITY_HUB),
            ("GetEnabledStandards", SECURITY_HUB),
            ("ListAutomationRules", SECURITY_HUB),
            ("ListFindingAggregators", SECURITY_HUB),
            ("ListMembers", SECURITY_HUB),
        ],
    },
    Family {
        service: "snow-device-management",
        version: "2021-08-04",
        status: 403,
        media_type: JSON,
        code: IAM_DENIED,
        original_class: ErrorClass::AccessDenied,
        captures: &[("ListDevices", br#"{"__type":"AccessDeniedException","message":"No devices found in this region."}"#)],
    },
    Family {
        service: "securitylake",
        version: "2018-05-10",
        status: 404,
        media_type: JSON,
        code: "ResourceNotFoundException",
        original_class: ErrorClass::Fatal,
        captures: &[("ListSubscribers", br#"{"__type":"ResourceNotFoundException","message":"The request failed because Security Lake isn't enabled for your account in any Regions. Enable Security Lake for your account and then try again."}"#)],
    },
    Family {
        service: "elasticache",
        version: "2015-02-02",
        status: 400,
        media_type: XML,
        code: "InvalidParameterValue",
        original_class: ErrorClass::Fatal,
        captures: &[("DescribeCacheSecurityGroups", b"<ErrorResponse><Error><Code>InvalidParameterValue</Code><Message>Use of cache security groups is not permitted in this API version for your account.</Message></Error></ErrorResponse>")],
    },
    Family {
        service: "redshift",
        version: "2012-12-01",
        status: 400,
        media_type: XML,
        code: "InvalidParameterValue",
        original_class: ErrorClass::Fatal,
        captures: &[("DescribeClusterSecurityGroups", b"<ErrorResponse><Error><Code>InvalidParameterValue</Code><Message>Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.</Message></Error></ErrorResponse>")],
    },
];

#[test]
fn given_reviewed_operation_when_captured_and_iam_errors_decode_then_only_capture_is_absent()
-> TestResult {
    for family in FAMILIES {
        let operations: Vec<_> = family
            .captures
            .iter()
            .map(|(operation, _)| *operation)
            .collect();
        let compiled = fixture::compile(family.service, family.version, &operations)
            .map_err(|error| format!("{}: {error}", family.service))?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        for &(operation, body) in family.captures {
            let row = format!("{} {operation}", family.service);
            let plan = catalog
                .lookup(family.service, operation)
                .ok_or_else(|| format!("{row}: plan absent"))?;
            let reason = format!(
                "AWS service failure: {:?} ({})",
                family.original_class, family.code
            );
            let response = wire(family.status, family.media_type, body)?;
            if matches!(family.service, "billingconductor" | "fms") {
                assert!(
                    matches!(validate_status(plan, response), Err(ApiError::Denied(actual)) if actual == reason),
                    "{row}: expected a denial"
                );
            } else {
                assert_absence(plan, response, &reason).map_err(|error| format!("{row}: {error}"))?;
            }
            assert_denials(family, plan, &row, &reason)?;
        }
    }
    Ok(())
}

#[test]
fn given_scoped_provider_messages_when_request_account_or_region_differs_then_denial_is_preserved()
-> TestResult {
    let account = "111111111111";
    let region = "us-west-2";
    let cases = [
        (
            "billingconductor",
            "ListBillingGroups",
            "Account 111111111111 is not authorized to call ListBillingGroups. Only payer account is authorized",
            false,
        ),
        (
            "fms",
            "ListPolicies",
            "No default admin could be found for account 111111111111 in Region us-west-2",
            true,
        ),
        (
            "securityhub",
            "DescribeHub",
            "Account 111111111111 is not subscribed to Security Hub CSPM in Region us-west-2",
            true,
        ),
    ];
    for (service, operation, message, names_region) in cases {
        let family = FAMILIES
            .iter()
            .find(|family| family.service == service)
            .ok_or("family absent")?;
        let compiled = fixture::compile(service, family.version, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or("plan absent")?;
        let reason = format!("AWS service failure: AccessDenied ({})", family.code);
        let classify = |account, region| -> TestResult<_> {
            let response = error_response(family, family.status, family.code, message)?;
            Ok(crate::call::validate_status(
                plan, response, account, region,
            ))
        };
        let matching = classify(account, region)?;
        if matches!(service, "billingconductor" | "fms") {
            assert!(
                matches!(matching, Err(ApiError::Denied(actual)) if actual == reason),
                "{service}:{operation} denial became empty coverage"
            );
        } else {
            assert!(
                matches!(matching.map(|response| response.decode(plan, false)), Ok(Err(SweepError::Absent(actual))) if actual == reason)
            );
        }
        let mismatches = [(ACCOUNT, region), (account, REGION)];
        for (account, region) in mismatches
            .into_iter()
            .take(if names_region { 2 } else { 1 })
        {
            assert!(
                matches!(classify(account, region)?, Err(ApiError::Denied(actual)) if actual == reason),
                "{service}: mismatching request {account}/{region}"
            );
        }
    }
    Ok(())
}

#[test]
fn given_near_match_fms_messages_when_decoded_then_denial_is_preserved() -> TestResult {
    let family = FAMILIES
        .iter()
        .find(|family| family.service == "fms")
        .ok_or("FMS family absent")?;
    let compiled = fixture::compile("fms", family.version, &["ListPolicies"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("fms", "ListPolicies").ok_or("plan absent")?;
    for message in [
        "No default admin could be found for account 123456789012 in Region us-east-1 (access denied)",
        "No default admin could be found for account 1234567890123 in Region us-east-1",
        "No default admin could be found for account 123456789012 in Region us-east-10",
    ] {
        let response = error_response(family, 400, IAM_DENIED, message)?;
        assert!(
            matches!(
                crate::call::validate_status(plan, response, "123456789012", "us-east-1"),
                Err(ApiError::Denied(reason))
                    if reason == "AWS service failure: AccessDenied (AccessDeniedException)"
            ),
            "{message}"
        );
    }
    Ok(())
}

#[test]
fn given_captured_absence_message_when_status_is_server_error_then_scope_is_not_empty() -> TestResult
{
    let compiled = fixture::compile("macie2", "2020-01-01", &["ListMembers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("macie2", "ListMembers")
        .ok_or("plan absent")?;
    assert!(matches!(
        validate_status(plan, wire(500, JSON, MACIE_ABSENT)?),
        Err(ApiError::Denied(reason))
            if reason == "AWS service failure: AccessDenied (AccessDeniedException)"
    ));
    Ok(())
}

fn assert_denials(family: &Family, plan: &CallablePlan<'_>, row: &str, reason: &str) -> TestResult {
    let operation = plan.operation_name();
    let service = plan.service_name();
    let message = format!(
        "User: arn:aws:iam::123456789012:role/r is not authorized to perform: {service}:{operation} with an explicit deny in a service control policy"
    );
    let denied = error_response(family, 403, IAM_DENIED, &message)?;
    assert!(
        matches!(validate_status(plan, denied), Err(ApiError::Denied(actual))
        if actual == "AWS service failure: AccessDenied (AccessDeniedException)"),
        "{row}: plain IAM denial changed outcome"
    );
    let same_code = error_response(family, family.status, family.code, &message)?;
    let preserved = match (family.original_class, validate_status(plan, same_code)) {
        (ErrorClass::AccessDenied, Err(ApiError::Denied(actual)))
        | (ErrorClass::Fatal, Err(ApiError::Permanent(actual))) => actual == reason,
        _ => false,
    };
    assert!(
        preserved,
        "{row}: same code with an unrelated message changed outcome"
    );
    Ok(())
}

fn error_response(family: &Family, status: u16, code: &str, message: &str) -> TestResult<Response> {
    let mut response = wire(status, family.media_type, b"")?;
    let bytes = match family.media_type {
        CBOR => {
            let mut encoded = minicbor::Encoder::new(Vec::new());
            encoded.map(2)?.str("__type")?.str(code)?.str("Message")?.str(message)?;
            encoded.into_writer()
        }
        XML => format!("<ErrorResponse><Error><Code>{code}</Code><Message>{message}</Message></Error></ErrorResponse>").into_bytes(),
        _ => serde_json::to_vec(&serde_json::json!({"__type": code, "message": message}))?,
    };
    response.body = bytes.into();
    Ok(response)
}
