// Pinned model: .local/aws-models/models/rum/service/2018-05-10/rum-2018-05-10.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListAppMonitors",
        items: "AppMonitorSummaries",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
