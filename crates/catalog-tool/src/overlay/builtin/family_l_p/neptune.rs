// Pinned Smithy source: .local/aws-models/models/neptune/service/2014-10-31/neptune-2014-10-31.json
use super::policy::{Group, Id, Input, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AddRoleToDBCluster",
            "AddSourceIdentifierToSubscription",
            "AddTagsToResource",
            "ApplyPendingMaintenanceAction",
            "CopyDBClusterParameterGroup",
            "CopyDBClusterSnapshot",
            "CopyDBParameterGroup",
            "CreateDBCluster",
            "CreateDBClusterEndpoint",
            "CreateDBClusterParameterGroup",
            "CreateDBClusterSnapshot",
            "CreateDBInstance",
            "CreateDBParameterGroup",
            "CreateDBSubnetGroup",
            "CreateEventSubscription",
            "CreateGlobalCluster",
            "DeleteDBCluster",
            "DeleteDBClusterEndpoint",
            "DeleteDBClusterParameterGroup",
            "DeleteDBClusterSnapshot",
            "DeleteDBInstance",
            "DeleteDBParameterGroup",
            "DeleteDBSubnetGroup",
            "DeleteEventSubscription",
            "DeleteGlobalCluster",
            "FailoverDBCluster",
            "FailoverGlobalCluster",
            "ModifyDBCluster",
            "ModifyDBClusterEndpoint",
            "ModifyDBClusterParameterGroup",
            "ModifyDBClusterSnapshotAttribute",
            "ModifyDBInstance",
            "ModifyDBParameterGroup",
            "ModifyDBSubnetGroup",
            "ModifyEventSubscription",
            "ModifyGlobalCluster",
            "PromoteReadReplicaDBCluster",
            "RebootDBInstance",
            "RemoveFromGlobalCluster",
            "RemoveRoleFromDBCluster",
            "RemoveSourceIdentifierFromSubscription",
            "RemoveTagsFromResource",
            "ResetDBClusterParameterGroup",
            "ResetDBParameterGroup",
            "RestoreDBClusterFromSnapshot",
            "RestoreDBClusterToPointInTime",
            "StartDBCluster",
            "StopDBCluster",
            "SwitchoverGlobalCluster",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeDBClusterParameters"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DBClusterParameterGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDBClusterSnapshotAttributes"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DBClusterSnapshotIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DescribeDBEngineVersions",
            "DescribeDBInstances",
            "DescribeEventCategories",
            "DescribeEvents",
            "DescribePendingMaintenanceActions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["DescribeDBParameters"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DBParameterGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DescribeEngineDefaultClusterParameters",
            "DescribeEngineDefaultParameters",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DBParameterGroupFamily); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeOrderableDBInstanceOptions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Engine); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeValidDBInstanceModifications"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DBInstanceIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns information about provisioned DB clusters, and supports pagination. This operation can also return information for Amazon RDS clusters and Amazon DocDB clusters.
    Resource {
        operation: "DescribeDBClusters",
        items: "DBClusters",
        id: Id::Arn("DBClusterArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "DBClusterArn",
            "DBClusterIdentifier",
            "Engine",
            "CharacterSetName",
            "DatabaseName",
            "Status",
            "MasterUsername",
            "HostedZoneId",
            "KmsKeyId",
            "DbClusterResourceId",
            "CloneGroupId",
            "StorageType",
            "NetworkType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[Input::Filter("Filters", "engine", &["neptune"])],
    },
    // Endpoints share the RDS API; bind only clusters discovered by the Neptune engine filter.
    Resource {
        operation: "DescribeDBClusterEndpoints",
        items: "DBClusterEndpoints",
        id: Id::Arn("DBClusterEndpointArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "DBClusterEndpointArn",
            "DBClusterIdentifier",
            "Status",
            "EndpointType",
            "CustomEndpointType",
        ],
        tags: None,
        bindings: &[(
            "DBClusterIdentifier",
            "DescribeDBClusters",
            "DBClusterIdentifier",
        )],
        fixed: &[],
    },
    // Returns a list of DBClusterParameterGroup descriptions. If a DBClusterParameterGroupName parameter is specified, the list will contain only the description of the specified DB clus
    Resource {
        operation: "DescribeDBClusterParameterGroups",
        items: "DBClusterParameterGroups",
        id: Id::Arn("DBClusterParameterGroupArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &["DBClusterParameterGroupArn", "DBClusterParameterGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of DBParameterGroup descriptions. If a DBParameterGroupName is specified, the list will contain only the description of the specified DB parameter group.
    Resource {
        operation: "DescribeDBParameterGroups",
        items: "DBParameterGroups",
        id: Id::Arn("DBParameterGroupArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &["DBParameterGroupArn", "DBParameterGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of DBSubnetGroup descriptions. If a DBSubnetGroupName is specified, the list will contain only the descriptions of the specified DBSubnetGroup. For an overview of CI
    Resource {
        operation: "DescribeDBSubnetGroups",
        items: "DBSubnetGroups",
        id: Id::Arn("DBSubnetGroupArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "DBSubnetGroupArn",
            "DBSubnetGroupName",
            "VpcId",
            "SubnetGroupStatus",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns information about DB cluster snapshots. This API action supports pagination.
    Resource {
        operation: "DescribeDBClusterSnapshots",
        items: "DBClusterSnapshots",
        id: Id::Arn("DBClusterSnapshotArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "DBClusterSnapshotArn",
            "Status",
            "VpcId",
            "MasterUsername",
            "SnapshotType",
            "KmsKeyId",
            "SourceDBClusterSnapshotArn",
            "StorageType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists all the subscription descriptions for a customer account. The description for a subscription includes SubscriptionName, SNSTopicARN, CustomerID, SourceType, SourceID, Creatio
    Resource {
        operation: "DescribeEventSubscriptions",
        items: "EventSubscriptionsList",
        id: Id::Arn("EventSubscriptionArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "EventSubscriptionArn",
            "CustomerAwsId",
            "CustSubscriptionId",
            "SnsTopicArn",
            "Status",
            "SourceType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns information about Neptune global database clusters. This API supports pagination.
    Resource {
        operation: "DescribeGlobalClusters",
        items: "GlobalClusters",
        id: Id::Arn("GlobalClusterArn"),
        scope: Scope::Region,
        tokens: Some(("Marker", "Marker")),
        query: &[
            "GlobalClusterArn",
            "GlobalClusterResourceId",
            "Status",
            "DatabaseName",
        ],
        tags: Some("TagList"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("neptune", GROUPS, RESOURCES)
}
