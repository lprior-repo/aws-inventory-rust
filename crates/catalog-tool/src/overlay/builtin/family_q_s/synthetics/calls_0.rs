use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListGroups",
    items: "Groups",
    identity: ResourceIdentity::Arn("Arn"),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Account,
    input: &[],
    tags: None,
    queryable: &[],
}];
