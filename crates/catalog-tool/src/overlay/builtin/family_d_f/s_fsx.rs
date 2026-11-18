// Pinned model: .local/aws-models/models/fsx/service/2018-03-01/fsx-2018-03-01.json
use super::support::{arn, inventory, key, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "fsx",
        vec![
            // Returns the description of a specific Amazon FSx backup, if a BackupIds value is provided for that backup.
            queried(
                tagged(
                    inventory(
                        "DescribeBackups",
                        "Backups",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "BackupId",
                    "Type",
                    "KmsKeyId",
                    "OwnerId",
                    "SourceBackupId",
                    "ResourceType",
                ],
            )?,
            // Returns the description of specific Amazon FSx for Lustre or Amazon File Cache data repository associations, if one or more AssociationIds values are provided in the request, or if
            queried(
                tagged(
                    inventory(
                        "DescribeDataRepositoryAssociations",
                        "Associations",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "AssociationId",
                    "FileSystemId",
                    "FileCacheId",
                ],
            )?,
            // Returns the description of a specific Amazon File Cache resource, if a FileCacheIds value is provided for that cache.
            queried(
                inventory(
                    "DescribeFileCaches",
                    "FileCaches",
                    arn("ResourceARN")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ResourceARN",
                    "OwnerId",
                    "FileCacheId",
                    "FileCacheType",
                    "FileCacheTypeVersion",
                    "VpcId",
                    "DNSName",
                    "KmsKeyId",
                ],
            )?,
            // Returns the description of specific Amazon FSx file systems, if a FileSystemIds value is provided for that file system.
            queried(
                tagged(
                    inventory(
                        "DescribeFileSystems",
                        "FileSystems",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "OwnerId",
                    "FileSystemId",
                    "FileSystemType",
                    "StorageType",
                    "VpcId",
                    "DNSName",
                    "KmsKeyId",
                    "FileSystemTypeVersion",
                    "NetworkType",
                ],
            )?,
            // Describes one or more S3 access points attached to Amazon FSx volumes.
            queried(
                inventory(
                    "DescribeS3AccessPointAttachments",
                    "S3AccessPointAttachments",
                    key(&["Name"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Name", "Type"],
            )?,
            // Returns the description of specific Amazon FSx for OpenZFS snapshots, if a SnapshotIds value is provided.
            queried(
                tagged(
                    inventory(
                        "DescribeSnapshots",
                        "Snapshots",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &["ResourceARN", "SnapshotId", "Name", "VolumeId"],
            )?,
            // Describes one or more Amazon FSx for NetApp ONTAP storage virtual machines (SVMs).
            queried(
                tagged(
                    inventory(
                        "DescribeStorageVirtualMachines",
                        "StorageVirtualMachines",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "FileSystemId",
                    "Name",
                    "StorageVirtualMachineId",
                    "Subtype",
                    "UUID",
                ],
            )?,
            // Describes one or more Amazon FSx for NetApp ONTAP or Amazon FSx for OpenZFS volumes.
            queried(
                tagged(
                    inventory(
                        "DescribeVolumes",
                        "Volumes",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "FileSystemId",
                    "Name",
                    "VolumeId",
                    "VolumeType",
                ],
            )?,
            // Metadata for existing repository tasks, not the exported files or report contents.
            queried(
                tagged(
                    inventory(
                        "DescribeDataRepositoryTasks",
                        "DataRepositoryTasks",
                        arn("ResourceARN")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceARN",
                    "TaskId",
                    "FileSystemId",
                    "FileCacheId",
                    "Lifecycle",
                    "Type",
                ],
            )?,
        ],
        &[
            super::s_fsx_decisions_0::ENTRIES,
            super::s_fsx_decisions_1::ENTRIES,
        ],
        &[],
    )
}
