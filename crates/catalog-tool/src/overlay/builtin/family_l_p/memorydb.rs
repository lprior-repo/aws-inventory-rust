// Pinned Smithy source: .local/aws-models/models/memorydb/service/2021-01-01/memorydb-2021-01-01.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "BatchUpdateCluster", "CopySnapshot", "CreateACL", "CreateCluster", 
            "CreateMultiRegionCluster", "CreateParameterGroup", "CreateSnapshot", 
            "CreateSubnetGroup", "CreateUser", "DeleteACL", "DeleteCluster", 
            "DeleteMultiRegionCluster", "DeleteParameterGroup", "DeleteSnapshot", 
            "DeleteSubnetGroup", "DeleteUser", "FailoverShard", 
            "PurchaseReservedNodesOffering", "ResetParameterGroup", "TagResource", 
            "UntagResource", "UpdateACL", "UpdateCluster", "UpdateMultiRegionCluster", 
            "UpdateParameterGroup", "UpdateSubnetGroup", "UpdateUser",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeEngineVersions", "DescribeEvents", "DescribeMultiRegionParameterGroups", 
            "DescribeReservedNodes", "DescribeReservedNodesOfferings", 
            "DescribeServiceUpdates",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "DescribeMultiRegionParameters",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (MultiRegionParameterGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeParameters",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ParameterGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAllowedMultiRegionClusterUpdates",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (MultiRegionClusterName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAllowedNodeTypeUpdates",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ClusterName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTags",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns information about all provisioned clusters if no cluster identifier is specified, or about a specific cluster if a cluster name is supplied.
    Resource {
        operation: "DescribeClusters",
        items: "Clusters", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "Status", "MultiRegionClusterName", "NodeType", "ParameterGroupName", "ParameterGroupStatus", "SubnetGroupName", "KmsKeyId", "SnsTopicArn", "SnsTopicStatus", "ACLName", "NetworkType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns details about one or more multi-Region clusters.
    Resource {
        operation: "DescribeMultiRegionClusters",
        items: "MultiRegionClusters", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "MultiRegionClusterName", "Status", "NodeType", "MultiRegionParameterGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of parameter group descriptions. If a parameter group name is specified, the list contains only the descriptions for that group.
    Resource {
        operation: "DescribeParameterGroups",
        items: "ParameterGroups", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of subnet group descriptions. If a subnet group name is specified, the list contains only the description of that group.
    Resource {
        operation: "DescribeSubnetGroups",
        items: "SubnetGroups", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "VpcId"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of ACLs.
    Resource {
        operation: "DescribeACLs",
        items: "ACLs", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of users.
    Resource {
        operation: "DescribeUsers",
        items: "Users", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns information about cluster snapshots. By default, DescribeSnapshots lists all of your snapshots; it can optionally describe a single snapshot, or just the snapshots associat
    Resource {
        operation: "DescribeSnapshots",
        items: "Snapshots", id: Id::Arn("ARN"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ARN", "Name", "Status", "KmsKeyId"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("memorydb", GROUPS, RESOURCES)
}
