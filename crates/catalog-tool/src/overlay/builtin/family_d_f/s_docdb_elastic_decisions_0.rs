// Explicit dispositions from .local/aws-models/models/docdb-elastic/service/2022-11-28/docdb-elastic-2022-11-28.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ApplyPendingMaintenanceAction",
        "Requires resourceArn, applyAction, optInType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The type of pending maintenance action to be applied to the resource.",
    ),
    (
        "CopyClusterSnapshot",
        "Requires snapshotArn, targetSnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Copies a snapshot of an elastic cluster.",
    ),
    (
        "CreateCluster",
        "Requires clusterName, authType, adminUserName, adminUserPassword, shardCapacity, shardCount; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new Amazon DocumentDB elastic cluster and returns its cluster structure.",
    ),
    (
        "CreateClusterSnapshot",
        "Requires clusterArn, snapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a snapshot of an elastic cluster.",
    ),
    (
        "DeleteCluster",
        "Requires clusterArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Delete an elastic cluster.",
    ),
    (
        "DeleteClusterSnapshot",
        "Requires snapshotArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Delete an elastic cluster snapshot.",
    ),
    (
        "GetCluster",
        "Requires clusterArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about a specific elastic cluster.",
    ),
    (
        "GetClusterSnapshot",
        "Requires snapshotArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about a specific elastic cluster snapshot",
    ),
    (
        "GetPendingMaintenanceAction",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves all maintenance actions that are pending.",
    ),
    (
        "ListPendingMaintenanceActions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves a list of all maintenance actions that are pending.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all tags on a elastic cluster resource",
    ),
    (
        "RestoreClusterFromSnapshot",
        "Requires clusterName, snapshotArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Restores an elastic cluster from a snapshot.",
    ),
    (
        "StartCluster",
        "Requires clusterArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Restarts the stopped elastic cluster that is specified by clusterARN .",
    ),
    (
        "StopCluster",
        "Requires clusterArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the running elastic cluster that is specified by clusterArn .",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds metadata tags to an elastic cluster resource",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes metadata tags from an elastic cluster resource",
    ),
    (
        "UpdateCluster",
        "Requires clusterArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an elastic cluster.",
    ),
];
