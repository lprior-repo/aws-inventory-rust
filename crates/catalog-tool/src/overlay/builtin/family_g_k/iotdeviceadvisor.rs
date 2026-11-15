//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iotdeviceadvisor/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListSuiteDefinitions", policy::query(policy::enumerate("suiteDefinitionInformationList", policy::key("suiteDefinitionId")?, Some(("nextToken", "nextToken")))?, &["suiteDefinitionId"])?),
    ];
    policy::finish(
        "iotdeviceadvisor", approved, DEFERRED,
        "Test-suite definitions are collected as summaries only. Test execution and remaining configuration/detail paths need separate binding/content review.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetEndpoint", "GetSuiteDefinition", "GetSuiteRun", "GetSuiteRunReport", "ListSuiteRuns",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateSuiteDefinition", "DeleteSuiteDefinition", "StartSuiteRun", "StopSuiteRun", "TagResource",
    "UntagResource", "UpdateSuiteDefinition",
];
