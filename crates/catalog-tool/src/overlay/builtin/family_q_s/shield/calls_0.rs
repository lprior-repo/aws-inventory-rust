use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListProtectionGroups",
        items: "ProtectionGroups",
        identity: ResourceIdentity::Arn("ProtectionGroupArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListProtections",
        items: "Protections",
        identity: ResourceIdentity::Arn("ProtectionArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListAttacks",
        items: "AttackSummaries",
        identity: ResourceIdentity::Key(&["AttackId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
