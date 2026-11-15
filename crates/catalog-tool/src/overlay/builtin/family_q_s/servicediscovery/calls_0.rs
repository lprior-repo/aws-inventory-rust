// Pinned model: .local/aws-models/models/servicediscovery/service/2017-03-14/servicediscovery-2017-03-14.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListServices",
        items: "Services",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListOperations",
        items: "Operations",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
