//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kinesis-analytics-v2/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListApplications",
            policy::query(
                policy::enumerate(
                    "ApplicationSummaries",
                    policy::arn("ApplicationARN")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["ApplicationARN"],
            )?,
        ),
        (
            "ListSnapshots",
            policy::query(
                policy::enumerate(
                    "SnapshotInfoList",
                    policy::arn("SnapshotArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["SnapshotArn"],
            )?,
        ),
    ];
    policy::finish(
        "kinesis-analytics-v2",
        approved,
        DEFERRED,
        "Application and enhanced snapshot summaries are metadata. Application detail/version and schema discovery can expose/evaluate application code/sample records and are denied. Per-application children and tags remain unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeApplicationOperation",
    "DescribeApplicationSnapshot",
    "DescribeSnapshot",
    "DescribeSnapshotOperation",
    "ListApplicationOperations",
    "ListApplicationSnapshots",
    "ListApplicationVersions",
    "ListSnapshotOperations",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AddApplicationCloudWatchLoggingOption",
    "AddApplicationInput",
    "AddApplicationInputProcessingConfiguration",
    "AddApplicationOutput",
    "AddApplicationReferenceDataSource",
    "AddApplicationVpcConfiguration",
    "CreateApplication",
    "CreateApplicationPresignedUrl",
    "CreateApplicationSnapshot",
    "CreateSnapshot",
    "DeleteApplication",
    "DeleteApplicationCloudWatchLoggingOption",
    "DeleteApplicationInputProcessingConfiguration",
    "DeleteApplicationOutput",
    "DeleteApplicationReferenceDataSource",
    "DeleteApplicationSnapshot",
    "DeleteApplicationVpcConfiguration",
    "DeleteSnapshot",
    "ExportSnapshot",
    "ImportSnapshot",
    "RollbackApplication",
    "StartApplication",
    "StopApplication",
    "TagResource",
    "UntagResource",
    "UpdateApplication",
    "UpdateApplicationMaintenanceConfiguration",
];

const CONTENT: &[&str] = &[
    "DescribeApplication",
    "DescribeApplicationVersion",
    "DiscoverInputSchema",
];
