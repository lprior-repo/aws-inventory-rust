// Pinned Smithy source: .local/aws-models/models/networkmonitor/service/2023-08-01/networkmonitor-2023-08-01.json
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
            "CreateProbe",
            "DeleteMonitor",
            "DeleteProbe",
            "TagResource",
            "UntagResource",
            "UpdateMonitor",
            "UpdateProbe",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetProbe"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (monitorName, probeId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
    // Returns a list of all of your monitors.
    Resource {
        operation: "ListMonitors",
        items: "monitors",
        id: Id::Arn("monitorArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["monitorArn", "monitorName", "state"],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetMonitor",
        items: "probes",
        id: Id::Arn("probeArn"),
        scope: Scope::Region,
        tokens: None,
        query: &["probeArn", "probeId", "sourceArn", "state"],
        tags: Some("tags"),
        bindings: &[("monitorName", "ListMonitors", "monitorName")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("networkmonitor", GROUPS, RESOURCES)
}
