// Pinned model: .local/aws-models/models/secrets-manager/service/2017-10-17/secrets-manager-2017-10-17.json
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
