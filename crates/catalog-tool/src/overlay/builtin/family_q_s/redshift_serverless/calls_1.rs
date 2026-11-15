// Pinned model: .local/aws-models/models/redshift-serverless/service/2021-04-21/redshift-serverless-2021-04-21.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListSnapshotCopyConfigurations",
        items: "snapshotCopyConfigurations",
        identity: ResourceIdentity::Arn("snapshotCopyConfigurationArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTableRestoreStatus",
        items: "tableRestoreStatuses",
        identity: ResourceIdentity::Key(&["tableRestoreRequestId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
