// Pinned model: .local/aws-models/models/security-ir/service/2018-05-10/security-ir-2018-05-10.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListMemberships",
        items: "items",
        identity: ResourceIdentity::Arn("membershipArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
