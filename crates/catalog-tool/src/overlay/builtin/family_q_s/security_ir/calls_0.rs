use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListMemberships",
    items: "items",
    identity: ResourceIdentity::Arn("membershipArn"),
    token: Some(("nextToken", "nextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
