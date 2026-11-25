use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateCliToken",
            "CreateEnvironment",
            "CreateWebLoginToken",
            "DeleteEnvironment",
            "InvokeRestApi",
            "PublishMetrics",
            "TagResource",
            "UntagResource",
            "UpdateEnvironment",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetEnvironment"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Environment summaries are safe; GetEnvironment exposes arbitrary Airflow configuration values that may contain credentials.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListEnvironments",
    items: "Environments",
    id: Id::Key(&["$"]),
    scope: Scope::Region,
    tokens: Some(("NextToken", "NextToken")),
    query: &["$"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mwaa", GROUPS, RESOURCES)
}
