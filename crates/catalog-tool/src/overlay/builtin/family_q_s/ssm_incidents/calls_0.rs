// Pinned model: .local/aws-models/models/ssm-incidents/service/2018-05-10/ssm-incidents-2018-05-10.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListResponsePlans",
        items: "responsePlanSummaries",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListReplicationSets",
        items: "replicationSetArns",
        identity: ResourceIdentity::Arn("$"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListIncidentRecords",
        items: "incidentRecordSummaries",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
