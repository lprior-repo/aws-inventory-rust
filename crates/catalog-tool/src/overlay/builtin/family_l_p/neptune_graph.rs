// Pinned Smithy source: .local/aws-models/models/neptune-graph/service/2023-11-29/neptune-graph-2023-11-29.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelExportTask",
            "CancelImportTask",
            "CancelQuery",
            "CreateGraph",
            "CreateGraphSnapshot",
            "CreateGraphUsingImportTask",
            "CreatePrivateGraphEndpoint",
            "DeleteGraph",
            "DeleteGraphSnapshot",
            "DeletePrivateGraphEndpoint",
            "ResetGraph",
            "RestoreGraphFromSnapshot",
            "StartExportTask",
            "StartGraph",
            "StartImportTask",
            "StopGraph",
            "TagResource",
            "UntagResource",
            "UpdateGraph",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["ExecuteQuery", "GetQuery"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetExportTask", "GetImportTask"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (taskIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetGraph", "GetGraphSummary"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (graphIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetGraphSnapshot"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (snapshotIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetPrivateGraphEndpoint"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (graphIdentifier, vpcId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListExportTasks", "ListImportTasks"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListQueries"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (graphIdentifier, maxResults); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
    // Lists available Neptune Analytics graphs.
    Resource {
        operation: "ListGraphs",
        items: "graphs",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists available snapshots of a specified Neptune Analytics graph.
    Resource {
        operation: "ListGraphSnapshots",
        items: "graphSnapshots",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name", "sourceGraphId", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListPrivateGraphEndpoints",
        items: "privateGraphEndpoints",
        id: Id::Correlated(&["graphIdentifier"], &["vpcId"]),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["vpcId", "vpcEndpointId", "status"],
        tags: None,
        bindings: &[("graphIdentifier", "ListGraphs", "id")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("neptune-graph", GROUPS, RESOURCES)
}
