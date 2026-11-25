use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![(
        "ListSuiteDefinitions",
        policy::query(
            policy::enumerate(
                "suiteDefinitionInformationList",
                policy::key("suiteDefinitionId")?,
                Some(("nextToken", "nextToken")),
            )?,
            &["suiteDefinitionId"],
        )?,
    )];
    policy::finish(
        "iotdeviceadvisor",
        approved,
        DEFERRED,
        "Test-suite definitions are collected as summaries only. Test execution and remaining configuration/detail paths need separate binding/content review.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[
    "GetEndpoint",
    "GetSuiteDefinition",
    "GetSuiteRun",
    "GetSuiteRunReport",
    "ListSuiteRuns",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateSuiteDefinition",
    "DeleteSuiteDefinition",
    "StartSuiteRun",
    "StopSuiteRun",
    "TagResource",
    "UntagResource",
    "UpdateSuiteDefinition",
];
