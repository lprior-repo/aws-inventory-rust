//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iotthingsgraph/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("SearchFlowTemplates", policy::query(policy::enumerate("summaries", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, &["arn"])?),
        ("SearchSystemTemplates", policy::query(policy::enumerate("summaries", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, &["arn"])?),
        ("SearchSystemInstances", policy::query(policy::enumerate("summaries", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, &["arn"])?),
    ];
    policy::finish(
        "iotthingsgraph", approved, DEFERRED,
        "Search summaries for flow/system templates and system instances are metadata, unlike their definition/document detail calls, which are denied. Namespaces and execution/dependent searches remain unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeNamespace", "GetFlowTemplateRevisions", "GetNamespaceDeletionStatus",
    "GetSystemTemplateRevisions", "GetUploadStatus", "ListTagsForResource", "SearchEntities",
    "SearchFlowExecutions", "SearchThings",
];

const MUTATING: &[&str] = &[
    "AssociateEntityToThing", "CreateFlowTemplate", "CreateSystemInstance", "CreateSystemTemplate",
    "DeleteFlowTemplate", "DeleteNamespace", "DeleteSystemInstance", "DeleteSystemTemplate",
    "DeploySystemInstance", "DeprecateFlowTemplate", "DeprecateSystemTemplate", "DissociateEntityFromThing",
    "TagResource", "UndeploySystemInstance", "UntagResource", "UpdateFlowTemplate", "UpdateSystemTemplate",
    "UploadEntityDefinitions",
];

const CONTENT: &[&str] = &[
    "GetEntities", "GetFlowTemplate", "GetSystemInstance", "GetSystemTemplate", "ListFlowExecutionMessages",
];
