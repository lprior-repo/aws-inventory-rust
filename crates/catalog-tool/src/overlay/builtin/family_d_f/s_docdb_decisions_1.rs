pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ModifyDBInstance",
        "Requires DBInstanceIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies settings for an instance.",
    ),
    (
        "ModifyDBSubnetGroup",
        "Requires DBSubnetGroupName, SubnetIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an existing subnet group.",
    ),
    (
        "ModifyEventSubscription",
        "Requires SubscriptionName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an existing Amazon DocumentDB event notification subscription.",
    ),
    (
        "ModifyGlobalCluster",
        "Requires GlobalClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modify a setting for an Amazon DocumentDB global cluster.",
    ),
    (
        "RebootDBInstance",
        "Requires DBInstanceIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: You might need to reboot your instance, usually for maintenance reasons.",
    ),
    (
        "RemoveFromGlobalCluster",
        "Requires GlobalClusterIdentifier, DbClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Detaches an Amazon DocumentDB secondary cluster from a global cluster.",
    ),
    (
        "RemoveSourceIdentifierFromSubscription",
        "Requires SubscriptionName, SourceIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a source identifier from an existing Amazon DocumentDB event notification subscription.",
    ),
    (
        "RemoveTagsFromResource",
        "Requires ResourceName, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes metadata tags from an Amazon DocumentDB resource.",
    ),
    (
        "ResetDBClusterParameterGroup",
        "Requires DBClusterParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the parameters of a cluster parameter group to the default value.",
    ),
    (
        "RestoreDBClusterFromSnapshot",
        "Requires DBClusterIdentifier, SnapshotIdentifier, Engine; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new cluster from a snapshot or cluster snapshot.",
    ),
    (
        "RestoreDBClusterToPointInTime",
        "Requires DBClusterIdentifier, SourceDBClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Restores a cluster to an arbitrary point in time.",
    ),
    (
        "StartDBCluster",
        "Requires DBClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Restarts the stopped cluster that is specified by DBClusterIdentifier .",
    ),
    (
        "StopDBCluster",
        "Requires DBClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the running cluster that is specified by DBClusterIdentifier .",
    ),
    (
        "SwitchoverGlobalCluster",
        "Requires GlobalClusterIdentifier, TargetDbClusterIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Switches over the specified secondary Amazon DocumentDB cluster to be the new primary Amazon DocumentDB cluster in the global database cluster.",
    ),
];
