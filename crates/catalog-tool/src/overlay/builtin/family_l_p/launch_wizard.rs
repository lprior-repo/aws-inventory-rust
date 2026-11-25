use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateDeployment",
            "DeleteDeployment",
            "TagResource",
            "UntagResource",
            "UpdateDeployment",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetDeployment", "ListDeploymentEvents"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (deploymentId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDeploymentPatternVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workloadName, deploymentPatternName, deploymentPatternVersionName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetWorkload", "ListWorkloadDeploymentPatterns"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workloadName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetWorkloadDeploymentPattern",
            "ListDeploymentPatternVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (workloadName, deploymentPatternName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
        operations: &["ListWorkloads"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListDeployments",
    items: "deployments",
    id: Id::Key(&["id"]),
    scope: Scope::Region,
    tokens: Some(("nextToken", "nextToken")),
    query: &["id", "name", "workloadName", "patternName", "status"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("launch-wizard", GROUPS, RESOURCES)
}
