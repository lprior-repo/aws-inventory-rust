// Pinned model: .local/aws-models/models/simpledbv2/service/2025-09-26/simpledbv2-2025-09-26.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListExports",
        items: "exportSummaries",
        identity: ResourceIdentity::Arn("exportArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
