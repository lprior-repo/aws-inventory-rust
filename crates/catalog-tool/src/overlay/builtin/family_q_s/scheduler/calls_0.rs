// Pinned model: .local/aws-models/models/scheduler/service/2021-06-30/scheduler-2021-06-30.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListScheduleGroups",
        items: "ScheduleGroups",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListSchedules",
        items: "Schedules",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
