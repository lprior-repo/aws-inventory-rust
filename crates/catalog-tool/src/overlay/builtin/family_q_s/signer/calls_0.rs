use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListSigningProfiles",
        items: "profiles",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Boolean("includeCanceled", true)],
        tags: Some("tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListSigningJobs",
        items: "jobs",
        identity: ResourceIdentity::Key(&["jobId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
