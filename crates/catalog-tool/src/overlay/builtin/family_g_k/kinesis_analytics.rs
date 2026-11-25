use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "kinesis-analytics",
        approved,
        DEFERRED,
        "Legacy ListApplications requires a last-item-derived ExclusiveStartApplicationName cursor with HasMoreApplications; no verified scalar-token continuation is available here. Application detail contains code. No callable policy is approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

const DEFERRED: &[&str] = &["ListApplications", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "AddApplicationCloudWatchLoggingOption",
    "AddApplicationInput",
    "AddApplicationInputProcessingConfiguration",
    "AddApplicationOutput",
    "AddApplicationReferenceDataSource",
    "CreateApplication",
    "DeleteApplication",
    "DeleteApplicationCloudWatchLoggingOption",
    "DeleteApplicationInputProcessingConfiguration",
    "DeleteApplicationOutput",
    "DeleteApplicationReferenceDataSource",
    "StartApplication",
    "StopApplication",
    "TagResource",
    "UntagResource",
    "UpdateApplication",
];

const CONTENT: &[&str] = &["DescribeApplication", "DiscoverInputSchema"];
