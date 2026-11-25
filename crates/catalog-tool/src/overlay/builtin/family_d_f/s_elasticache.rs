use super::support::{arn, inventory, queried, reviewed, scoped, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elasticache",
        vec![
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
            queried(
                inventory(
                    "DescribeCacheParameterGroups",
                    "CacheParameterGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "CacheParameterGroupName"],
            )?,
            queried(
                inventory(
                    "DescribeCacheSecurityGroups",
                    "CacheSecurityGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "OwnerId", "CacheSecurityGroupName"],
            )?,
            queried(
                inventory(
                    "DescribeCacheSubnetGroups",
                    "CacheSubnetGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "CacheSubnetGroupName", "VpcId"],
            )?,
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
            queried(
                inventory(
                    "DescribeUserGroups",
                    "UserGroups",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "UserGroupId", "Status"],
            )?,
            queried(
                inventory(
                    "DescribeUsers",
                    "Users",
                    arn("ARN")?,
                    token("Marker", "Marker")?,
                )?,
                &["ARN", "UserId", "UserName", "Status"],
            )?,
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
