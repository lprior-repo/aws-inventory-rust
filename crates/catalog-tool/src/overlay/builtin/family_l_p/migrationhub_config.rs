use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["CreateHomeRegionControl", "DeleteHomeRegionControl"],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetHomeRegion"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "DescribeHomeRegionControls",
    items: "HomeRegionControls",
    id: Id::Key(&["ControlId"]),
    scope: Scope::HomeRegion,
    tokens: Some(("NextToken", "NextToken")),
    query: &["ControlId", "HomeRegion"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("migrationhub-config", GROUPS, RESOURCES)
}
