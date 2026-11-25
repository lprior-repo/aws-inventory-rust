use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateMonitor",
            "CreateScope",
            "DeleteMonitor",
            "DeleteScope",
            "StartQueryMonitorTopContributors",
            "StartQueryWorkloadInsightsTopContributors",
            "StartQueryWorkloadInsightsTopContributorsData",
            "StopQueryMonitorTopContributors",
            "StopQueryWorkloadInsightsTopContributors",
            "StopQueryWorkloadInsightsTopContributorsData",
            "TagResource",
            "UntagResource",
            "UpdateMonitor",
            "UpdateScope",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetMonitor"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (monitorName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetQueryResultsMonitorTopContributors",
            "GetQueryStatusMonitorTopContributors",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (monitorName, queryId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetQueryResultsWorkloadInsightsTopContributors",
            "GetQueryResultsWorkloadInsightsTopContributorsData",
            "GetQueryStatusWorkloadInsightsTopContributors",
            "GetQueryStatusWorkloadInsightsTopContributorsData",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (scopeId, queryId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetScope"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (scopeId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListMonitors",
        items: "monitors",
        id: Id::Arn("monitorArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["monitorArn", "monitorName", "monitorStatus"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListScopes",
        items: "scopes",
        id: Id::Arn("scopeArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["scopeArn", "scopeId", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("networkflowmonitor", GROUPS, RESOURCES)
}
