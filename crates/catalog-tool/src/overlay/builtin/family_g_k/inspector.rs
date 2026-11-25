use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListAssessmentTargets",
            policy::query(
                policy::enumerate(
                    "assessmentTargetArns",
                    policy::arn("$")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["$"],
            )?,
        ),
        (
            "ListAssessmentTemplates",
            policy::query(
                policy::enumerate(
                    "assessmentTemplateArns",
                    policy::arn("$")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["$"],
            )?,
        ),
    ];
    policy::finish(
        "inspector",
        approved,
        DEFERRED,
        "Assessment target/template ARN listings are metadata. Batch detail APIs report failedItems and need explicit completeness handling before approval; findings, exclusions and run traversal are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[
    "DescribeAssessmentRuns",
    "DescribeAssessmentTargets",
    "DescribeAssessmentTemplates",
    "DescribeCrossAccountAccessRole",
    "DescribeExclusions",
    "DescribeFindings",
    "DescribeResourceGroups",
    "DescribeRulesPackages",
    "GetAssessmentReport",
    "GetExclusionsPreview",
    "GetTelemetryMetadata",
    "ListAssessmentRunAgents",
    "ListAssessmentRuns",
    "ListEventSubscriptions",
    "ListExclusions",
    "ListFindings",
    "ListRulesPackages",
    "ListTagsForResource",
    "PreviewAgents",
];

const MUTATING: &[&str] = &[
    "AddAttributesToFindings",
    "CreateAssessmentTarget",
    "CreateAssessmentTemplate",
    "CreateExclusionsPreview",
    "CreateResourceGroup",
    "DeleteAssessmentRun",
    "DeleteAssessmentTarget",
    "DeleteAssessmentTemplate",
    "RegisterCrossAccountAccessRole",
    "RemoveAttributesFromFindings",
    "SetTagsForResource",
    "StartAssessmentRun",
    "StopAssessmentRun",
    "SubscribeToEvent",
    "UnsubscribeFromEvent",
    "UpdateAssessmentTarget",
];
