// Pinned Smithy source: .local/aws-models/models/notifications/service/2018-05-10/notifications-2018-05-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateChannel", "AssociateManagedNotificationAccountContact", 
            "AssociateManagedNotificationAdditionalChannel", "AssociateOrganizationalUnit", 
            "CreateEventRule", "CreateNotificationConfiguration", "DeleteEventRule", 
            "DeleteNotificationConfiguration", "DeregisterNotificationHub", 
            "DisableNotificationsAccessForOrganization", "DisassociateChannel", 
            "DisassociateManagedNotificationAccountContact", 
            "DisassociateManagedNotificationAdditionalChannel", 
            "DisassociateOrganizationalUnit", "EnableNotificationsAccessForOrganization", 
            "RegisterNotificationHub", "TagResource", "UntagResource", "UpdateEventRule", 
            "UpdateManagedNotificationChannelAssociation", "UpdateNotificationConfiguration",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetEventRule", "GetManagedNotificationChildEvent", 
            "GetManagedNotificationConfiguration", "GetManagedNotificationEvent", 
            "GetNotificationConfiguration", "GetNotificationEvent", "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetNotificationsAccessForOrganization", "ListManagedNotificationEvents", 
            "ListNotificationEvents",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListChannels", "ListMemberAccounts", "ListOrganizationalUnits",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (notificationConfigurationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListManagedNotificationChannelAssociations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (managedNotificationConfigurationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListManagedNotificationChildEvents",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (aggregateManagedNotificationEventArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of abbreviated NotificationConfigurations according to specified filters, in reverse chronological order (newest first).
    Resource {
        operation: "ListNotificationConfigurations",
        items: "notificationConfigurations", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "status", "subtype"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of Managed Notification Configurations according to specified filters, ordered by creation time in reverse chronological order (newest first).
    Resource {
        operation: "ListManagedNotificationConfigurations",
        items: "managedNotificationConfigurations", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of NotificationHubs .
    Resource {
        operation: "ListNotificationHubs",
        items: "notificationHubs", id: Id::Key(&["notificationHubRegion"]),
        scope: Scope::Account, tokens: Some(("nextToken", "nextToken")),
        query: &["notificationHubRegion"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListEventRules", items: "eventRules", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "notificationConfigurationArn", "source", "eventType"], tags: None,
        bindings: &[("notificationConfigurationArn", "ListNotificationConfigurations", "arn")], fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("notifications", GROUPS, RESOURCES)
}
