// Explicit dispositions from .local/aws-models/models/elasticache/service/2015-02-02/elasticache-2015-02-02.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AddTagsToResource",
        "Requires ResourceName, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: A tag is a key-value pair where the key and value are case-sensitive.",
    ),
    (
        "AuthorizeCacheSecurityGroupIngress",
        "Requires CacheSecurityGroupName, EC2SecurityGroupName, EC2SecurityGroupOwnerId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows network ingress to a cache security group.",
    ),
    (
        "BatchApplyUpdateAction",
        "Requires ServiceUpdateName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Apply the service update.",
    ),
    (
        "BatchStopUpdateAction",
        "Requires ServiceUpdateName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stop the service update.",
    ),
    (
        "CompleteMigration",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Completes the migration of data.",
    ),
    (
        "CopyServerlessCacheSnapshot",
        "Requires SourceServerlessCacheSnapshotName, TargetServerlessCacheSnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a copy of an existing serverless cache’s snapshot.",
    ),
    (
        "CopySnapshot",
        "Requires SourceSnapshotName, TargetSnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Makes a copy of an existing snapshot.",
    ),
    (
        "CreateCacheCluster",
        "Requires CacheClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a cluster.",
    ),
    (
        "CreateCacheParameterGroup",
        "Requires CacheParameterGroupName, CacheParameterGroupFamily, Description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new Amazon ElastiCache cache parameter group.",
    ),
    (
        "CreateCacheSecurityGroup",
        "Requires CacheSecurityGroupName, Description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new cache security group.",
    ),
    (
        "CreateCacheSubnetGroup",
        "Requires CacheSubnetGroupName, CacheSubnetGroupDescription, SubnetIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new cache subnet group.",
    ),
    (
        "CreateGlobalReplicationGroup",
        "Requires GlobalReplicationGroupIdSuffix, PrimaryReplicationGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Global Datastore offers fully managed, fast, reliable and secure cross-region replication.",
    ),
    (
        "CreateReplicationGroup",
        "Requires ReplicationGroupId, ReplicationGroupDescription; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Valkey or Redis OSS (cluster mode disabled) or a Valkey or Redis OSS (cluster mode enabled) replication group.",
    ),
    (
        "CreateServerlessCache",
        "Requires ServerlessCacheName, Engine; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a serverless cache.",
    ),
    (
        "CreateServerlessCacheSnapshot",
        "Requires ServerlessCacheSnapshotName, ServerlessCacheName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This API creates a copy of an entire ServerlessCache at a specific moment in time.",
    ),
    (
        "CreateSnapshot",
        "Requires SnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a copy of an entire cluster or replication group at a specific moment in time.",
    ),
    (
        "CreateUser",
        "Requires UserId, UserName, Engine, AccessString; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For Valkey engine version 7.2 onwards and Redis OSS 6.0 to 7.1: Creates a user.",
    ),
    (
        "CreateUserGroup",
        "Requires UserGroupId, Engine; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For Valkey engine version 7.2 onwards and Redis OSS 6.0 to 7.1: Creates a user group.",
    ),
    (
        "DecreaseNodeGroupsInGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, NodeGroupCount, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Decreases the number of node groups in a Global datastore",
    ),
    (
        "DecreaseReplicaCount",
        "Requires ReplicationGroupId, ApplyImmediately; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Dynamically decreases the number of replicas in a Valkey or Redis OSS (cluster mode disabled) replication group or the number of replica nodes in one or more node groups (shards) of a Valkey or Redis ",
    ),
    (
        "DeleteCacheCluster",
        "Requires CacheClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a previously provisioned cluster.",
    ),
    (
        "DeleteCacheParameterGroup",
        "Requires CacheParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified cache parameter group.",
    ),
    (
        "DeleteCacheSecurityGroup",
        "Requires CacheSecurityGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a cache security group.",
    ),
    (
        "DeleteCacheSubnetGroup",
        "Requires CacheSubnetGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a cache subnet group.",
    ),
    (
        "DeleteGlobalReplicationGroup",
        "Requires GlobalReplicationGroupId, RetainPrimaryReplicationGroup; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deleting a Global datastore is a two-step process: First, you must DisassociateGlobalReplicationGroup to remove the secondary clusters in the Global datastore.",
    ),
    (
        "DeleteReplicationGroup",
        "Requires ReplicationGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an existing replication group.",
    ),
    (
        "DeleteServerlessCache",
        "Requires ServerlessCacheName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a specified existing serverless cache.",
    ),
    (
        "DeleteServerlessCacheSnapshot",
        "Requires ServerlessCacheSnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an existing serverless cache snapshot.",
    ),
    (
        "DeleteSnapshot",
        "Requires SnapshotName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an existing snapshot.",
    ),
    (
        "DeleteUser",
        "Requires UserId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For Valkey engine version 7.2 onwards and Redis OSS 6.0 onwards: Deletes a user.",
    ),
    (
        "DeleteUserGroup",
        "Requires UserGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For Valkey engine version 7.2 onwards and Redis OSS 6.0 onwards: Deletes a user group.",
    ),
    (
        "DescribeCacheEngineVersions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns a list of the available cache engines and their versions.",
    ),
    (
        "DescribeCacheParameters",
        "Requires CacheParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the detailed parameter list for a particular cache parameter group.",
    ),
    (
        "DescribeEngineDefaultParameters",
        "Requires CacheParameterGroupFamily; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the default engine and system parameter information for the specified cache engine.",
    ),
    (
        "DescribeEvents",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns events related to clusters, cache security groups, and cache parameter groups.",
    ),
];
