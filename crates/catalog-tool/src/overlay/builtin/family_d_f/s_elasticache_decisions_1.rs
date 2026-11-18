// Explicit dispositions from .local/aws-models/models/elasticache/service/2015-02-02/elasticache-2015-02-02.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DescribeReservedCacheNodesOfferings",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists available reserved cache node offerings.",
    ),
    (
        "DescribeServiceUpdates",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns details of the service updates",
    ),
    (
        "DescribeUpdateActions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns details of the update actions",
    ),
    (
        "DisassociateGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, ReplicationGroupId, ReplicationGroupRegion; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Remove a secondary cluster from the Global datastore using the Global datastore name.",
    ),
    (
        "ExportServerlessCacheSnapshot",
        "Requires ServerlessCacheSnapshotName, S3BucketName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provides the functionality to export the serverless cache snapshot data to Amazon S3.",
    ),
    (
        "FailoverGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, PrimaryRegion, PrimaryReplicationGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Used to failover the primary region to a secondary region.",
    ),
    (
        "IncreaseNodeGroupsInGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, NodeGroupCount, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Increase the number of node groups in the Global datastore",
    ),
    (
        "IncreaseReplicaCount",
        "Requires ReplicationGroupId, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Dynamically increases the number of replicas in a Valkey or Redis OSS (cluster mode disabled) replication group or the number of replica nodes in one or more node groups (shards) of a Valkey or Redis ",
    ),
    (
        "ListAllowedNodeTypeModifications",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists all available node types that you can scale with your cluster's replication group's current node type.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all tags currently on a named resource.",
    ),
    (
        "ModifyCacheCluster",
        "Requires CacheClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the settings for a cluster.",
    ),
    (
        "ModifyCacheParameterGroup",
        "Requires CacheParameterGroupName, ParameterNameValues; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the parameters of a cache parameter group.",
    ),
    (
        "ModifyCacheSubnetGroup",
        "Requires CacheSubnetGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an existing cache subnet group.",
    ),
    (
        "ModifyGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the settings for a Global datastore.",
    ),
    (
        "ModifyReplicationGroup",
        "Requires ReplicationGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the settings for a replication group.",
    ),
    (
        "ModifyReplicationGroupShardConfiguration",
        "Requires ReplicationGroupId, NodeGroupCount, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies a replication group's shards (node groups) by allowing you to add shards, remove shards, or rebalance the keyspaces among existing shards.",
    ),
    (
        "ModifyServerlessCache",
        "Requires ServerlessCacheName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This API modifies the attributes of a serverless cache.",
    ),
    (
        "ModifyUser",
        "Requires UserId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Changes user password(s) and/or access string.",
    ),
    (
        "ModifyUserGroup",
        "Requires UserGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Changes the list of users that belong to the user group.",
    ),
    (
        "PrepareMigration",
        "Requires ServerlessCacheName, CustomerNodeEndpointList; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Prepares the serverless cache for data migration by validating the configuration and setting up the migration resources.",
    ),
    (
        "PurchaseReservedCacheNodesOffering",
        "Requires ReservedCacheNodesOfferingId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to purchase a reserved cache node offering.",
    ),
    (
        "RebalanceSlotsInGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Redistribute slots to ensure uniform distribution across existing shards in the cluster.",
    ),
    (
        "RebootCacheCluster",
        "Requires CacheClusterId, CacheNodeIdsToReboot; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Reboots some, or all, of the cache nodes within a provisioned cluster.",
    ),
    (
        "RemoveTagsFromResource",
        "Requires ResourceName, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the tags identified by the TagKeys list from the named resource.",
    ),
    (
        "ResetCacheParameterGroup",
        "Requires CacheParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the parameters of a cache parameter group to the engine or system default value.",
    ),
    (
        "RevokeCacheSecurityGroupIngress",
        "Requires CacheSecurityGroupName, EC2SecurityGroupName, EC2SecurityGroupOwnerId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Revokes ingress from a cache security group.",
    ),
    (
        "StartMigration",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Starts the migration of data.",
    ),
    (
        "StopMigration",
        "Requires ServerlessCacheName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops an ongoing data sync between the source and the serverless cache.",
    ),
    (
        "TestFailover",
        "Requires ReplicationGroupId, NodeGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Represents the input of a TestFailover operation which tests automatic failover on a specified node group (called shard in the console) in a replication group (called cluster in the console).",
    ),
    (
        "TestMigration",
        "Requires ReplicationGroupId, CustomerNodeEndpointList; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Async API to test connection between source and target replication group.",
    ),
];
