use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListApplications",
        items: "Applications",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListComponents",
        items: "Components",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListDatabases",
        items: "Databases",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
];
