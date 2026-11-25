use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListSecrets",
    items: "SecretList",
    identity: ResourceIdentity::Arn("ARN"),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Region,
    input: &[Input::Boolean("IncludePlannedDeletion", true)],
    tags: Some("Tags"),
    queryable: &[],
}];
