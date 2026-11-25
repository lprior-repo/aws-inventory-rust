use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListTableBuckets",
        items: "tableBuckets",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("continuationToken", "continuationToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTables",
        items: "tables",
        identity: ResourceIdentity::Arn("tableARN"),
        token: Some(("continuationToken", "continuationToken")),
        scope: Scope::Region,
        input: &[Input::Parent("tableBucketARN", "ListTableBuckets", "arn")],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTableMigrations",
        items: "migrations",
        identity: ResourceIdentity::Key(&["migrationId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
