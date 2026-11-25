use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListSpaces",
    items: "spaces",
    identity: ResourceIdentity::Arn("arn"),
    token: Some(("nextToken", "nextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
