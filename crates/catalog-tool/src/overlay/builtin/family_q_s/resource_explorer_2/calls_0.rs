use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListViews",
        items: "Views",
        identity: ResourceIdentity::Arn("$"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "GetIndex",
        items: "$",
        identity: ResourceIdentity::Arn("Arn"),
        token: None,
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListStreamingAccessForServices",
        items: "StreamingAccessForServices",
        identity: ResourceIdentity::Key(&["ServicePrincipal"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
