use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateAct",
            "CreateSession",
            "CreateWorkflowDefinition",
            "CreateWorkflowRun",
            "DeleteWorkflowDefinition",
            "DeleteWorkflowRun",
            "InvokeActStep",
            "UpdateAct",
            "UpdateWorkflowRun",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetWorkflowDefinition", "ListActs", "ListWorkflowRuns"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workflowDefinitionName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkflowRun", "ListSessions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workflowDefinitionName, workflowRunId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListModels"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (clientCompatibilityVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListWorkflowDefinitions",
    items: "workflowDefinitionSummaries",
    id: Id::Arn("workflowDefinitionArn"),
    scope: Scope::Region,
    tokens: Some(("nextToken", "nextToken")),
    query: &["workflowDefinitionArn", "workflowDefinitionName", "status"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("nova-act", GROUPS, RESOURCES)
}
