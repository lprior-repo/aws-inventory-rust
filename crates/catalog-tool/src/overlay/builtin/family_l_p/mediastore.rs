use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateContainer",
            "DeleteContainer",
            "DeleteContainerPolicy",
            "DeleteCorsPolicy",
            "DeleteLifecyclePolicy",
            "DeleteMetricPolicy",
            "PutContainerPolicy",
            "PutCorsPolicy",
            "PutLifecyclePolicy",
            "PutMetricPolicy",
            "StartAccessLogging",
            "StopAccessLogging",
            "TagResource",
            "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetContainerPolicy",
            "GetCorsPolicy",
            "GetLifecyclePolicy",
            "GetMetricPolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ContainerName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Resource); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListContainers",
        items: "Containers",
        id: Id::Arn("ARN"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "DescribeContainer",
        items: "Container",
        id: Id::Arn("ARN"),
        scope: Scope::Region,
        tokens: None,
        query: &["ARN", "Name", "Status"],
        tags: None,
        bindings: &[("ContainerName", "ListContainers", "Name")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediastore", GROUPS, RESOURCES)
}
