// Pinned model: .local/aws-models/models/s3vectors/service/2025-07-15/s3vectors-2025-07-15.json
use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListVectorBuckets",
        items: "vectorBuckets",
        identity: ResourceIdentity::Arn("vectorBucketArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListIndexes",
        items: "indexes",
        identity: ResourceIdentity::Arn("indexArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Parent(
            "vectorBucketArn",
            "ListVectorBuckets",
            "vectorBucketArn",
        )],
        tags: None,
        queryable: &[],
    },
];
