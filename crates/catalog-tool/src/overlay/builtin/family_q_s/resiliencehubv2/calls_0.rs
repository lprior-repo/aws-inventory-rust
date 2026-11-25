use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListSystems",
        items: "systemSummaries",
        identity: ResourceIdentity::Key(&["systemId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListServices",
        items: "serviceSummaries",
        identity: ResourceIdentity::Arn("serviceArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Account("accountId")],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListPolicies",
        items: "policySummaries",
        identity: ResourceIdentity::Arn("policyArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Account("accountId")],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListDependencies",
        items: "dependencySummaries",
        identity: ResourceIdentity::Key(&["serviceArn", "dependencyId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
