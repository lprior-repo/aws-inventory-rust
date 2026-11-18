// Pinned model: .local/aws-models/models/efs/service/2015-02-01/efs-2015-02-01.json
use super::support::{arn, bound, inventory, key, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "efs",
        vec![
            // Returns the description of a specific Amazon EFS file system if either the file system CreationToken or the FileSystemId is provided.
            queried(
                tagged(
                    inventory(
                        "DescribeFileSystems",
                        "FileSystems",
                        arn("FileSystemArn")?,
                        token("Marker", "NextMarker")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "FileSystemArn",
                    "OwnerId",
                    "FileSystemId",
                    "LifeCycleState",
                    "Name",
                    "KmsKeyId",
                    "AvailabilityZoneName",
                    "AvailabilityZoneId",
                ],
            )?,
            // Returns the description of a specific Amazon EFS access point if the AccessPointId is provided.
            queried(
                tagged(
                    bound(
                        inventory(
                            "DescribeAccessPoints",
                            "AccessPoints",
                            arn("AccessPointArn")?,
                            token("NextToken", "NextToken")?,
                        )?,
                        &[("FileSystemId", "DescribeFileSystems", "FileSystemId")],
                    )?,
                    "Tags",
                )?,
                &[
                    "AccessPointArn",
                    "Name",
                    "AccessPointId",
                    "FileSystemId",
                    "OwnerId",
                    "LifeCycleState",
                ],
            )?,
            // Retrieves the replication configuration for a specific file system.
            queried(
                inventory(
                    "DescribeReplicationConfigurations",
                    "Replications",
                    arn("SourceFileSystemArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "SourceFileSystemArn",
                    "SourceFileSystemId",
                    "SourceFileSystemOwnerId",
                ],
            )?,
            // Returns the descriptions of all the current mount targets, or a specific mount target, for a file system.
            queried(
                bound(
                    inventory(
                        "DescribeMountTargets",
                        "MountTargets",
                        key(&["MountTargetId"])?,
                        token("Marker", "NextMarker")?,
                    )?,
                    &[("FileSystemId", "DescribeFileSystems", "FileSystemId")],
                )?,
                &[
                    "MountTargetId",
                    "OwnerId",
                    "FileSystemId",
                    "SubnetId",
                    "LifeCycleState",
                    "NetworkInterfaceId",
                    "AvailabilityZoneId",
                    "AvailabilityZoneName",
                    "VpcId",
                ],
            )?,
        ],
        &[super::s_efs_decisions_0::ENTRIES],
        &[],
    )
}
