// Pinned model: .local/aws-models/models/securitylake/service/2018-05-10/securitylake-2018-05-10.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListSubscribers",
        items: "subscribers",
        identity: ResourceIdentity::Arn("subscriberArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
