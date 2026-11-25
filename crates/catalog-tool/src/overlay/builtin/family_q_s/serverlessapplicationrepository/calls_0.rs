use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListApplications",
    items: "Applications",
    identity: ResourceIdentity::Arn("ApplicationId"),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
