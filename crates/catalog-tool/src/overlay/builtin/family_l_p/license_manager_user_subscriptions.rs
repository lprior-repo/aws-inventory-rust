use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateUser",
            "CreateLicenseServerEndpoint",
            "DeleteLicenseServerEndpoint",
            "DeregisterIdentityProvider",
            "DisassociateUser",
            "RegisterIdentityProvider",
            "StartProductSubscription",
            "StopProductSubscription",
            "TagResource",
            "UntagResource",
            "UpdateIdentityProviderSettings",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["ListProductSubscriptions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (IdentityProvider); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListUserAssociations"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InstanceId, IdentityProvider); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListIdentityProviders",
        items: "IdentityProviderSummaries",
        id: Id::Arn("IdentityProviderArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["IdentityProviderArn", "Status", "OwnerAccountId"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListInstances",
        items: "InstanceSummaries",
        id: Id::Key(&["OwnerAccountId", "InstanceId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["OwnerAccountId", "InstanceId", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListLicenseServerEndpoints",
        items: "LicenseServerEndpoints",
        id: Id::Arn("LicenseServerEndpointArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "LicenseServerEndpointArn",
            "IdentityProviderArn",
            "ServerType",
            "LicenseServerEndpointId",
            "LicenseServerEndpointProvisioningStatus",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("license-manager-user-subscriptions", GROUPS, RESOURCES)
}
