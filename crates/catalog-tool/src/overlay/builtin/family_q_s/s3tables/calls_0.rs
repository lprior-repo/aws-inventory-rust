// Pinned model: .local/aws-models/models/s3tables/service/2018-05-10/s3tables-2018-05-10.json
use super::super::construction::{Inventory, ResourceIdentity, Input};
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
