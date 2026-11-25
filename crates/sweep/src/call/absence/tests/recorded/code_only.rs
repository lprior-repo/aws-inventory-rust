use super::*;

struct Capture {
    status: u16,
    content_type: &'static str,
    headers: &'static [(&'static str, &'static str)],
    body: &'static [u8],
}

impl Capture {
    fn response(&self) -> TestResult<Response> {
        let mut response = wire(self.status, self.content_type, self.body)?;
        for &(name, value) in self.headers {
            response
                .headers
                .append(name, http::HeaderValue::from_static(value));
        }
        Ok(response)
    }
}

const S3_CONTROL: &[(&str, Capture)] = &[
    (
        "GetAccessGrantsInstance",
        Capture {
            status: 404,
            content_type: "application/xml",
            headers: &[
                ("x-amz-request-id", "CBF8886D1FB53DF1"),
                ("x-amz-id-2", "e94e38903921a4d7c4f1ed97635856c491a4abd513e740198961bee5907662fc"),
            ],
            body: br#"<ErrorResponse xmlns="http://awss3control.amazonaws.com/doc/2018-08-20/"><Error><Type>Sender</Type><Code>AccessGrantsInstanceNotExistsError</Code><Message>Access Grants Instance does not exist</Message></Error><RequestId>CBF8886D1FB53DF1</RequestId><HostId>e94e38903921a4d7c4f1ed97635856c491a4abd513e740198961bee5907662fc</HostId></ErrorResponse>"#,
        },
    ),
    (
        "ListAccessGrants",
        Capture {
            status: 404,
            content_type: "application/xml",
            headers: &[
                ("x-amz-request-id", "E79819075F8AF694"),
                ("x-amz-id-2", "81e499cca49780113290a59c42ba4cf2628f38dd1480eebc9f48abb335b61933"),
            ],
            body: br#"<ErrorResponse xmlns="http://awss3control.amazonaws.com/doc/2018-08-20/"><Error><Type>Sender</Type><Code>AccessGrantsInstanceNotExistsError</Code><Message>Access Grants Instance does not exist</Message></Error><RequestId>E79819075F8AF694</RequestId><HostId>81e499cca49780113290a59c42ba4cf2628f38dd1480eebc9f48abb335b61933</HostId></ErrorResponse>"#,
        },
    ),
    (
        "ListAccessGrantsLocations",
        Capture {
            status: 404,
            content_type: "application/xml",
            headers: &[
                ("x-amz-request-id", "34E63C1C48C5DEA6"),
                ("x-amz-id-2", "a2ae2f22d7bb0ccd6e814460358737146870ed25d0feed34d7acce5d889a1f51"),
            ],
            body: br#"<ErrorResponse xmlns="http://awss3control.amazonaws.com/doc/2018-08-20/"><Error><Type>Sender</Type><Code>AccessGrantsInstanceNotExistsError</Code><Message>Access Grants Instance does not exist</Message></Error><RequestId>34E63C1C48C5DEA6</RequestId><HostId>a2ae2f22d7bb0ccd6e814460358737146870ed25d0feed34d7acce5d889a1f51</HostId></ErrorResponse>"#,
        },
    ),
];

const DRS: Capture = Capture {
    status: 400,
    content_type: "application/json",
    headers: &[("x-amzn-errortype", "UninitializedAccountException")],
    body: br#"{"__type":"UninitializedAccountException","message":"Elastic Disaster Recovery has not been initialized for this account; call InitializeService first"}"#,
};
const DRS_REASON: &str = "AWS service failure: Fatal (UninitializedAccountException)";

#[test]
fn given_recorded_s3_instance_errors_when_decoded_then_only_instance_reads_are_absent() -> TestResult
{
    let compiled = fixture::compile(
        "s3-control",
        "2018-08-20",
        &[
            "GetAccessGrantsInstance",
            "ListAccessGrants",
            "ListAccessGrantsLocations",
            "GetAccessGrant",
        ],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let detail = catalog
        .lookup("s3-control", "GetAccessGrant")
        .ok_or("detail plan absent")?;
    let reason = "AWS service failure: Fatal (AccessGrantsInstanceNotExistsError)";
    for (operation, captured) in S3_CONTROL {
        let plan = catalog.lookup("s3-control", operation).ok_or(*operation)?;
        assert_absence(plan, captured.response()?, reason)?;
        assert!(matches!(
            validate_status(detail, captured.response()?),
            Err(ApiError::Permanent(actual)) if actual == reason
        ));
    }
    Ok(())
}

#[test]
fn given_recorded_uninitialized_drs_when_status_changes_then_only_client_error_is_absent()
-> TestResult {
    let compiled = fixture::compile("drs", "2020-02-26", &["DescribeSourceServers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("drs", "DescribeSourceServers")
        .ok_or("plan absent")?;
    assert_absence(plan, DRS.response()?, DRS_REASON)?;
    let mut unavailable = DRS.response()?;
    unavailable.status = http::StatusCode::SERVICE_UNAVAILABLE;
    assert!(matches!(
        validate_status(plan, unavailable),
        Err(ApiError::Transient(reason)) if reason == DRS_REASON
    ));
    let mut redirect = DRS.response()?;
    redirect.status = http::StatusCode::FOUND;
    assert!(matches!(
        validate_status(plan, redirect),
        Err(ApiError::Permanent(reason)) if reason == DRS_REASON
    ));
    Ok(())
}

#[test]
fn given_recorded_preexisting_absence_errors_when_decoded_then_scope_is_absent() -> TestResult {
    for (service, version, operations, operation, reason, captured) in PREEXISTING {
        let compiled = fixture::compile(service, version, operations)?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or(*operation)?;
        assert_absence(plan, captured.response()?, reason)?;
    }
    Ok(())
}

type Case = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
    &'static str,
    Capture,
);
const PREEXISTING: &[Case] = &[
    (
        "iam", "2010-05-08", &["ListUsers", "GetLoginProfile"], "GetLoginProfile",
        "AWS service failure: Fatal (NoSuchEntity)",
        Capture {
            status: 404,
            content_type: "text/xml",
            headers: &[("x-amzn-requestid", "35f8dc20-0208-48d6-a23c-f8833a8b226f")],
            body: br#"<ErrorResponse xmlns="https://iam.amazonaws.com/doc/2010-05-08/"><Error><Type>Sender</Type><Code>NoSuchEntity</Code><Message>Login Profile for User recorded-absence cannot be found.</Message></Error><RequestId>35f8dc20-0208-48d6-a23c-f8833a8b226f</RequestId></ErrorResponse>"#,
        },
    ),
    (
        "mgn", "2020-02-26", &["DescribeSourceServers"], "DescribeSourceServers",
        DRS_REASON,
        Capture {
            status: 400,
            content_type: "application/json",
            headers: &[("x-amzn-errortype", "UninitializedAccountException")],
            body: br#"{"__type":"UninitializedAccountException","message":"Account not initialized"}"#,
        },
    ),
    (
        "ec2", "2016-11-15", &["DescribeSpotDatafeedSubscription"], "DescribeSpotDatafeedSubscription",
        "AWS service failure: Fatal (InvalidSpotDatafeed.NotFound)",
        Capture {
            status: 400,
            content_type: "text/xml;charset=UTF-8",
            headers: &[],
            body: br#"<Response xmlns="http://ec2.amazonaws.com/doc/2016-11-15"><Errors><Error><Code>InvalidSpotDatafeed.NotFound</Code><Message>Spot datafeed subscription does not exist.</Message></Error></Errors><RequestID>00000000-0000-0000-0000-000000000001</RequestID></Response>"#,
        },
    ),
    (
        "resource-explorer-2", "2022-07-28", &["GetIndex"], "GetIndex",
        "AWS service failure: Fatal (ResourceNotFoundException)",
        Capture {
            status: 404,
            content_type: "application/json",
            headers: &[("x-amzn-errortype", "ResourceNotFoundException")],
            body: br#"{"__type":"ResourceNotFoundException","Message":"Index not found in Region us-east-1"}"#,
        },
    ),
    (
        "compute-optimizer", "2019-11-01", &["DescribeRecommendationExportJobs"], "DescribeRecommendationExportJobs",
        "AWS service failure: AccessDenied (OptInRequiredException)",
        Capture {
            status: 403,
            content_type: "application/x-amz-json-1.0",
            headers: &[],
            body: br#"{"__type":"OptInRequiredException","message":"The account is not opted in to Compute Optimizer."}"#,
        },
    ),
    (
        "compute-optimizer-automation", "2025-09-22", &["ListAutomationRules"], "ListAutomationRules",
        "AWS service failure: AccessDenied (OptInRequiredException)",
        Capture {
            status: 403,
            content_type: "application/x-amz-json-1.0",
            headers: &[],
            body: br#"{"__type":"OptInRequiredException","message":"Aws account is not registered for recommendation."}"#,
        },
    ),
];
