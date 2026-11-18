// Pinned model: .local/aws-models/models/qbusiness/service/2023-11-27/qbusiness-2023-11-27.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListApplications",
    items: "applications",
    identity: ResourceIdentity::Key(&["applicationId"]),
    token: Some(("nextToken", "nextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
