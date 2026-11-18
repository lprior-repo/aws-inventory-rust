// Pinned model: .local/aws-models/models/elasticache/service/2015-02-02/elasticache-2015-02-02.json
use super::support::{arn, inventory, queried, reviewed, scoped, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elasticache",
        vec![
            // Returns information about all provisioned clusters if no cluster identifier is specified, or about a specific cache cluster if a cluster identifier is supplied.
            queried(
                inventory(
                    "DescribeCacheClusters",
                    "CacheClusters",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ARN",
                    "CacheClusterId",
                    "CacheNodeType",
                    "CacheClusterStatus",
                    "CacheSubnetGroupName",
                    "ReplicationGroupId",
                    "NetworkType",
                ],
            )?,
            // Returns a list of cache parameter group descriptions.
            queried(
                inventory(
                    "DescribeCacheParameterGroups",
                    "CacheParameterGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "CacheParameterGroupName"],
            )?,
            // Returns a list of cache security group descriptions.
            queried(
                inventory(
                    "DescribeCacheSecurityGroups",
                    "CacheSecurityGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "OwnerId", "CacheSecurityGroupName"],
            )?,
            // Returns a list of cache subnet group descriptions.
            queried(
                inventory(
                    "DescribeCacheSubnetGroups",
                    "CacheSubnetGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "CacheSubnetGroupName", "VpcId"],
            )?,
            // Returns information about a particular global replication group.
            queried(
                scoped(
                    inventory(
                        "DescribeGlobalReplicationGroups",
                        "GlobalReplicationGroups",
                        arn("ARN")?,
                        token("Marker", "Marker")?,
                    )?,
                    Scope::Account,
                ),
                &["ARN", "GlobalReplicationGroupId", "Status", "CacheNodeType"],
            )?,
            // Returns information about a particular replication group.
            queried(
                inventory(
                    "DescribeReplicationGroups",
                    "ReplicationGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ARN",
                    "ReplicationGroupId",
                    "Status",
                    "SnapshottingClusterId",
                    "CacheNodeType",
                    "KmsKeyId",
                    "StorageEncryptionType",
                    "NetworkType",
                ],
            )?,
            // Returns information about a specific serverless cache.
            queried(
                inventory(
                    "DescribeServerlessCaches",
                    "ServerlessCaches",
                    arn("ARN")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ARN",
                    "ServerlessCacheName",
                    "Status",
                    "KmsKeyId",
                    "StorageEncryptionType",
                    "UserGroupId",
                    "NetworkType",
                    "ConnectionType",
                ],
            )?,
            // Returns information about serverless cache snapshots.
            queried(
                inventory(
                    "DescribeServerlessCacheSnapshots",
                    "ServerlessCacheSnapshots",
                    arn("ARN")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ARN",
                    "ServerlessCacheSnapshotName",
                    "KmsKeyId",
                    "SnapshotType",
                    "Status",
                ],
            )?,
            // Returns information about cluster or replication group snapshots.
            queried(
                inventory(
                    "DescribeSnapshots",
                    "Snapshots",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ARN",
                    "SnapshotName",
                    "ReplicationGroupId",
                    "CacheClusterId",
                    "SnapshotStatus",
                    "CacheNodeType",
                    "CacheParameterGroupName",
                    "CacheSubnetGroupName",
                    "VpcId",
                    "KmsKeyId",
                ],
            )?,
            // Returns a list of user groups.
            queried(
                inventory(
                    "DescribeUserGroups",
                    "UserGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "UserGroupId", "Status"],
            )?,
            // Returns a list of users.
            queried(
                inventory(
                    "DescribeUsers",
                    "Users",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "UserId", "UserName", "Status"],
            )?,
            // Existing reservations are metadata; this does not purchase or enumerate public offerings.
            queried(
                inventory(
                    "DescribeReservedCacheNodes",
                    "ReservedCacheNodes",
                    arn("ReservationARN")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ReservationARN",
                    "ReservedCacheNodeId",
                    "State",
                    "CacheNodeType",
                ],
            )?,
        ],
        &[
            super::s_elasticache_decisions_0::ENTRIES,
            super::s_elasticache_decisions_1::ENTRIES,
        ],
        &[],
    )
}
