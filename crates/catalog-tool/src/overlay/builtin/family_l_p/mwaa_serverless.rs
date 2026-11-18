// Pinned Smithy source: .local/aws-models/models/mwaa-serverless/service/2024-07-26/mwaa-serverless-2024-07-26.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateAirflowUiUrl",
            "CreateWorkflow",
            "DeleteWorkflow",
            "StartWorkflowRun",
            "StopWorkflowRun",
            "TagResource",
            "UntagResource",
            "UpdateWorkflow",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetTaskInstance"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (WorkflowArn, TaskInstanceId, RunId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkflow", "ListWorkflowRuns", "ListWorkflowVersions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (WorkflowArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkflowRun", "ListTaskInstances"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (WorkflowArn, RunId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists all workflows in your account, with optional pagination support. This operation returns summary information for workflows, showing only the most recently created version of e
    Resource {
        operation: "ListWorkflows",
        items: "Workflows",
        id: Id::Arn("WorkflowArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["WorkflowArn", "Name", "WorkflowStatus"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mwaa-serverless", GROUPS, RESOURCES)
}
