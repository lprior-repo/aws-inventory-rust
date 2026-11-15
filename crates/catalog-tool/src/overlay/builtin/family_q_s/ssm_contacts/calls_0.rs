// Pinned model: .local/aws-models/models/ssm-contacts/service/2021-05-03/ssm-contacts-2021-05-03.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListContacts",
        items: "Contacts",
        identity: ResourceIdentity::Arn("ContactArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListRotations",
        items: "Rotations",
        identity: ResourceIdentity::Arn("RotationArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListEngagements",
        items: "Engagements",
        identity: ResourceIdentity::Arn("EngagementArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
