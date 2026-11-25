use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListAppMonitors",
    items: "AppMonitorSummaries",
    identity: ResourceIdentity::Key(&["Id"]),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
