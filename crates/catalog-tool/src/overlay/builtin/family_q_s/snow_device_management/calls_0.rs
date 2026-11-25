use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListDevices",
        items: "devices",
        identity: ResourceIdentity::Arn("managedDeviceArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListTasks",
        items: "tasks",
        identity: ResourceIdentity::Arn("taskArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &[],
    },
];
