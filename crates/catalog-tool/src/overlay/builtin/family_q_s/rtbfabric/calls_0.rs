// Pinned model: .local/aws-models/models/rtbfabric/service/2023-05-15/rtbfabric-2023-05-15.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListRequesterGateways",
        items: "gatewayIds",
        identity: ResourceIdentity::Key(&["$"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListResponderGateways",
        items: "gatewayIds",
        identity: ResourceIdentity::Key(&["$"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
