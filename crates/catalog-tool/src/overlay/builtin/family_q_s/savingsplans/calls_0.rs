use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "DescribeSavingsPlans",
    items: "savingsPlans",
    identity: ResourceIdentity::Arn("savingsPlanArn"),
    token: Some(("nextToken", "nextToken")),
    scope: Scope::Account,
    input: &[],
    tags: Some("tags"),
    queryable: &[],
}];
