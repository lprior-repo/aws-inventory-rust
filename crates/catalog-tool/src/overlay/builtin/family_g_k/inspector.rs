//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/inspector/service.
//! Review is source-only; no live service or emulator support claim.
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

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
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
