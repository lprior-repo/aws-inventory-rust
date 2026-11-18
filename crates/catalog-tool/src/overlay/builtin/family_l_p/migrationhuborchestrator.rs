// Pinned Smithy source: .local/aws-models/models/migrationhuborchestrator/service/2021-08-28/migrationhuborchestrator-2021-08-28.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateTemplate",
            "CreateWorkflow",
            "CreateWorkflowStep",
            "CreateWorkflowStepGroup",
            "DeleteTemplate",
            "DeleteWorkflow",
            "DeleteWorkflowStep",
            "DeleteWorkflowStepGroup",
            "RetryWorkflowStep",
            "StartWorkflow",
            "StopWorkflow",
            "TagResource",
            "UntagResource",
            "UpdateTemplate",
            "UpdateWorkflow",
            "UpdateWorkflowStep",
            "UpdateWorkflowStepGroup",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetTemplate", "GetWorkflow"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplateStep"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id, templateId, stepGroupId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplateStepGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateId, id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkflowStep"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workflowId, stepGroupId, id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkflowStepGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id, workflowId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListPlugins", "ListTemplates"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTemplateStepGroups"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTemplateSteps"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateId, stepGroupId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListWorkflowStepGroups"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workflowId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListWorkflowSteps"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workflowId, stepGroupId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // List the migration workflows.
    Resource {
        operation: "ListWorkflows",
        items: "migrationWorkflowSummary",
        id: Id::Key(&["id"]),
        scope: Scope::HomeRegion,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "id",
            "name",
            "templateId",
            "adsApplicationConfigurationName",
            "status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("migrationhuborchestrator", GROUPS, RESOURCES)
}
