// Pinned model: .local/aws-models/models/ram/service/2018-01-04/ram-2018-01-04.json
use super::super::construction::{Inventory, ResourceIdentity, Input};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "GetResourceShares",
        items: "resourceShares",
        identity: ResourceIdentity::Arn("resourceShareArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::String("resourceOwner", "SELF")],
        tags: Some("tags"),
        queryable: &[],
    },
    Inventory {
        operation: "GetResourceShareInvitations",
        items: "resourceShareInvitations",
        identity: ResourceIdentity::Arn("resourceShareInvitationArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListPermissionAssociations",
        items: "permissions",
        identity: ResourceIdentity::Key(&["resourceShareArn", "arn", "permissionVersion"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListSourceAssociations",
        items: "sourceAssociations",
        identity: ResourceIdentity::Key(&["resourceShareArn", "sourceId", "sourceType"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListReplacePermissionAssociationsWork",
        items: "replacePermissionAssociationsWorks",
        identity: ResourceIdentity::Key(&["id"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
