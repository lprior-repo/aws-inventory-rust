// Pinned model: .local/aws-models/models/supplychain/service/2024-01-01/supplychain-2024-01-01.json
use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListInstances",
        items: "instances",
        identity: ResourceIdentity::Key(&["instanceId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListDataLakeNamespaces",
        items: "namespaces",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Parent("instanceId", "ListInstances", "instanceId")],
        tags: None,
        queryable: &["instanceId", "name"],
    },
    Inventory {
        operation: "ListDataLakeDatasets",
        items: "datasets",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[
            Input::Parent("instanceId", "ListDataLakeNamespaces", "instanceId"),
            Input::Parent("namespace", "ListDataLakeNamespaces", "name"),
        ],
        tags: None,
        queryable: &[],
    },
];
