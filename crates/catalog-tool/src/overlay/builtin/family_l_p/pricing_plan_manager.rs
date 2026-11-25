use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["ApprovePaidSubscription"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (arn, ifMatch); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "AssociateResourcesToSubscription",
            "CancelSubscription",
            "CancelSubscriptionChange",
            "CreateSubscription",
            "DisassociateResourcesFromSubscription",
            "UpdateSubscription",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetSubscription"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (arn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListSubscriptions",
    items: "subscriptionSummaries",
    id: Id::Arn("arn"),
    scope: Scope::Account,
    tokens: Some(("nextToken", "nextToken")),
    query: &["arn", "status"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pricing-plan-manager", GROUPS, RESOURCES)
}
