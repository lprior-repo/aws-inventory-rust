// Pinned model: .local/aws-models/models/s3files/service/2025-05-05/s3files-2025-05-05.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListFileSystems",
        items: "fileSystems",
        identity: ResourceIdentity::Arn("fileSystemArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListMountTargets",
        items: "mountTargets",
        identity: ResourceIdentity::Key(&["mountTargetId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
