use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "DeregisterSubscriptionProvider",
            "RegisterSubscriptionProvider",
            "TagResource",
            "UntagResource",
            "UpdateServiceSettings",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetRegisteredSubscriptionProvider"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (SubscriptionProviderArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetServiceSettings"],
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
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListLinuxSubscriptionInstances",
        items: "Instances",
        id: Id::Key(&["AccountID", "Region", "InstanceID"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "AccountID",
            "Region",
            "InstanceID",
            "AmiId",
            "InstanceType",
            "Status",
            "SubscriptionName",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListLinuxSubscriptions",
        items: "Subscriptions",
        id: Id::Key(&["Name", "Type"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Name", "Type"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListRegisteredSubscriptionProviders",
        items: "RegisteredSubscriptionProviders",
        id: Id::Arn("SubscriptionProviderArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "SubscriptionProviderArn",
            "SecretArn",
            "SubscriptionProviderStatus",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("license-manager-linux-subscriptions", GROUPS, RESOURCES)
}
