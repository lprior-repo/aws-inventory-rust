use super::support::{arn, bound, inventory, key, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "efs",
        vec![
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
