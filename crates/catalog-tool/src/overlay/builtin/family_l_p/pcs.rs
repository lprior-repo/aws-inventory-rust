// Pinned Smithy source: .local/aws-models/models/pcs/service/2023-02-10/pcs-2023-02-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateCluster", "CreateComputeNodeGroup", "CreateQueue", "DeleteCluster", 
            "DeleteComputeNodeGroup", "DeleteQueue", "RegisterComputeNodeGroupInstance", 
            "TagResource", "UntagResource", "UpdateCluster", "UpdateComputeNodeGroup", 
            "UpdateQueue",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetCluster",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (clusterIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetComputeNodeGroup",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (clusterIdentifier, computeNodeGroupIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetQueue",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (clusterIdentifier, queueIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of running clusters in your account.
    Resource {
        operation: "ListClusters",
        items: "clusters", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "id", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of all compute node groups associated with a cluster.
    Resource {
        operation: "ListComputeNodeGroups",
        items: "computeNodeGroups", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "id", "clusterId", "status"],
        tags: None,
        bindings: &[("clusterIdentifier", "ListClusters", "id")],
        fixed: &[],
    },
    // Returns a list of all queues associated with a cluster.
    Resource {
        operation: "ListQueues",
        items: "queues", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "id", "clusterId", "status"],
        tags: None,
        bindings: &[("clusterIdentifier", "ListClusters", "id")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pcs", GROUPS, RESOURCES)
}
