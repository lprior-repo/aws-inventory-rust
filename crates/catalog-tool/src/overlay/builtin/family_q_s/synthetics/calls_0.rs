// Pinned model: .local/aws-models/models/synthetics/service/2017-10-11/synthetics-2017-10-11.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListGroups",
        items: "Groups",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
