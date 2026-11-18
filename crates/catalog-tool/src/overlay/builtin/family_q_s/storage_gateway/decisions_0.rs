// Pinned operation purposes: .local/aws-models/models/storage-gateway/service/2013-06-30/storage-gateway-2013-06-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ActivateGateway",
        Decision::Deferred(
            "Required selectors ActivationKey, GatewayName, GatewayTimezone, GatewayRegion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN. Pinned purpose: Activates the gateway you previously deployed on your host. In the activation process, you specify information such as the Amazon Web Services Region that you want to use for storing snapshots or tapes, the time zone for scheduled snapshots the gatew",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Configures one or more gateway local disks as cache for a gateway. This operation is only supported in the cached volume, tape, and file gateway type (see How Storage Gateway works (architecture) . In the request, you specify the gateway Amazon Resource Name (ARN) to which you want to add cache, and one or more disk IDs that you want to configure as cache.
    ("AddCache", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds one or more tags to the specified resource. You use tags to add metadata to resources, which you can use to categorize these resources. For example, you can categorize resources by purpose, owner, environment, or team. Each tag consists of a key and a value, which you define. You can add tags to the following Storage Gateway resources: Storage gateways of all types Storage volumes Virtual tap
    ("AddTagsToResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Configures one or more gateway local disks as upload buffer for a specified gateway. This operation is supported for the stored volume, cached volume, and tape gateway types. In the request, you specify the gateway Amazon Resource Name (ARN) to which you want to add upload buffer, and one or more disk IDs that you want to configure as upload buffer.
    ("AddUploadBuffer", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Configures one or more gateway local disks as working storage for a gateway. This operation is only supported in the stored volume gateway type. This operation is deprecated in cached volume API version 20120630. Use AddUploadBuffer instead. Working storage is also referred to as upload buffer. You can also use the AddUploadBuffer operation to add upload buffer to a stored volume gateway. In the r
    ("AddWorkingStorage", Decision::Denied(DenyReason::Mutating)),
    (
        "AssignTapePool",
        Decision::Deferred(
            "Required selectors TapeARN, PoolId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TapeARN. Pinned purpose: Assigns a tape to a tape pool for archiving. The tape assigned to a pool is archived in the S3 storage class that is associated with the pool. When you use your backup application to eject the tape, the tape is archived directly into the S3 storage c",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associate an Amazon FSx file system with the FSx File Gateway. After the association process is complete, the file shares on the Amazon FSx file system are available for access through the gateway. This operation only supports the FSx File Gateway type.
    (
        "AssociateFileSystem",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Connects a volume to an iSCSI connection and then attaches the volume to the specified gateway. Detaching and attaching a volume enables you to recover your data from one gateway to a different gateway without creating a snapshot. It also makes it easier to move your volumes from an on-premises gateway to a gateway hosted on an Amazon EC2 instance.
    ("AttachVolume", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels archiving of a virtual tape to the virtual tape shelf (VTS) after the archiving process is initiated. This operation is only supported in the tape gateway type.
    ("CancelArchival", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels generation of a specified cache report. You can use this operation to manually cancel an IN-PROGRESS report for any reason. This action changes the report status from IN-PROGRESS to CANCELLED. You can only cancel in-progress reports. If the the report you attempt to cancel is in FAILED, ERROR, or COMPLETED state, the cancel operation returns an error.
    ("CancelCacheReport", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels retrieval of a virtual tape from the virtual tape shelf (VTS) to a gateway after the retrieval process is initiated. The virtual tape is returned to the VTS. This operation is only supported in the tape gateway type.
    ("CancelRetrieval", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a cached volume on a specified cached volume gateway. This operation is only supported in the cached volume gateway type. Cache storage must be allocated to the gateway before you can create a cached volume. Use the AddCache operation to add cache storage to a gateway. In the request, you must specify the gateway, size of the volume in bytes, the iSCSI target name, an IP address on which t
    (
        "CreateCachediSCSIVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Network File System (NFS) file share on an existing S3 File Gateway. In Storage Gateway, a file share is a file system mount point backed by Amazon S3 cloud storage. Storage Gateway exposes file shares using an NFS interface. This operation is only supported for S3 File Gateways. S3 File gateway requires Security Token Service (Amazon Web Services STS) to be activated to enable you to cr
    ("CreateNFSFileShare", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Server Message Block (SMB) file share on an existing S3 File Gateway. In Storage Gateway, a file share is a file system mount point backed by Amazon S3 cloud storage. Storage Gateway exposes file shares using an SMB interface. This operation is only supported for S3 File Gateways. S3 File Gateways require Security Token Service (Amazon Web Services STS) to be activated to enable you to c
    ("CreateSMBFileShare", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates a snapshot of a volume. Storage Gateway provides the ability to back up point-in-time snapshots of your data to Amazon Simple Storage (Amazon S3) for durable off-site recovery, and also import the data to an Amazon Elastic Block Store (EBS) volume in Amazon Elastic Compute Cloud (EC2). You can take snapshots of your gateway volume on a scheduled or ad hoc basis. This API enables you to t
    ("CreateSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates a snapshot of a gateway from a volume recovery point. This operation is only supported in the cached volume gateway type. A volume recovery point is a point in time at which all data of the volume is consistent and from which you can create a snapshot. To get a list of volume recovery point for cached volume gateway, use ListVolumeRecoveryPoints . In the CreateSnapshotFromVolumeRecoveryP
    (
        "CreateSnapshotFromVolumeRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a volume on a specified gateway. This operation is only supported in the stored volume gateway type. The size of the volume to create is inferred from the disk size. You can choose to preserve existing data on the disk, create volume from an existing snapshot, or create an empty volume. If you choose to create an empty gateway volume, then any existing data on the disk is erased. In the re
    (
        "CreateStorediSCSIVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new custom tape pool. You can use custom tape pool to enable tape retention lock on tapes that are archived in the custom pool.
    ("CreateTapePool", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a virtual tape by using your own barcode. You write data to the virtual tape and then archive the tape. A barcode is unique and cannot be reused if it has already been used on a tape. This applies to barcodes used on deleted tapes. This operation is only supported in the tape gateway type. Cache storage must be allocated to the gateway before you can create a virtual tape. Use the AddCache
    (
        "CreateTapeWithBarcode",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates one or more virtual tapes. You write data to the virtual tapes and then archive the tapes. This operation is only supported in the tape gateway type. Cache storage must be allocated to the gateway before you can create virtual tapes. Use the AddCache operation to add cache storage to a gateway.
    ("CreateTapes", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the automatic tape creation policy of a gateway. If you delete this policy, new virtual tapes must be created manually. Use the Amazon Resource Name (ARN) of the gateway in your request to remove the policy.
    (
        "DeleteAutomaticTapeCreationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the bandwidth rate limits of a gateway. You can delete either the upload and download bandwidth rate limit, or you can delete both. If you delete only one of the limits, the other limit remains unchanged. To specify which gateway to work with, use the Amazon Resource Name (ARN) of the gateway in your request. This operation is supported only for the stored volume, cached volume, and tape g
    (
        "DeleteBandwidthRateLimit",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified cache report and any associated tags from the Storage Gateway database. You can only delete completed reports. If the status of the report you attempt to delete still IN-PROGRESS, the delete operation returns an error. You can use CancelCacheReport to cancel an IN-PROGRESS report. DeleteCacheReport does not delete the report object from your Amazon S3 bucket.
    ("DeleteCacheReport", Decision::Denied(DenyReason::Mutating)),
    // Full modeled response contains authentication/secret material . Pinned purpose: Deletes Challenge-Handshake Authentication Protocol (CHAP) credentials for a specified iSCSI target and initiator pair. This operation is supported in volume and tape gateway types.. Projection cannot authorize this call.
    (
        "DeleteChapCredentials",
        Decision::Denied(DenyReason::Secret),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a file share from an S3 File Gateway. This operation is only supported for S3 File Gateways.
    ("DeleteFileShare", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a gateway. To specify which gateway to delete, use the Amazon Resource Name (ARN) of the gateway in your request. The operation deletes the gateway; however, it does not delete the gateway virtual machine (VM) from your host computer. After you delete a gateway, you cannot reactivate it. Completed snapshots of the gateway volumes are not deleted upon deleting the gateway, however, pending
    ("DeleteGateway", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a snapshot of a volume. You can take snapshots of your gateway volumes on a scheduled or ad hoc basis. This API action enables you to delete a snapshot schedule for a volume. For more information, see Backing up your volumes . In the DeleteSnapshotSchedule request, you identify the volume by providing its Amazon Resource Name (ARN). This operation is only supported for cached volume gatewa
    (
        "DeleteSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified virtual tape. This operation is only supported in the tape gateway type.
    ("DeleteTape", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified virtual tape from the virtual tape shelf (VTS). This operation is only supported in the tape gateway type.
    ("DeleteTapeArchive", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a custom tape pool. A custom tape pool can only be deleted if there are no tapes in the pool and if there are no automatic tape creation policies that reference the custom tape pool.
    ("DeleteTapePool", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified storage volume that you previously created using the CreateCachediSCSIVolume or CreateStorediSCSIVolume API. This operation is only supported in the cached volume and stored volume types. For stored volume gateways, the local disk that was configured as the storage volume is not deleted. You can reuse the local disk to create another storage volume. Before you delete a volume
    ("DeleteVolume", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeAvailabilityMonitorTest",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, Status, StartTime. Pinned purpose: Returns information about the most recent high availability monitoring test that was performed on the host in a cluster. If a test isn't performed, the status and start time in the response would be null.",
        ),
    ),
    (
        "DescribeBandwidthRateLimit",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, AverageUploadRateLimitInBitsPerSec, AverageDownloadRateLimitInBitsPerSec. Pinned purpose: Returns the bandwidth rate limits of a gateway. By default, these limits are not set, which means no bandwidth rate limiting is in effect. This operation is supported only for the stored volume, cached volume, and tape gateway types. To describe band",
        ),
    ),
    (
        "DescribeBandwidthRateLimitSchedule",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, BandwidthRateLimitIntervals. Pinned purpose: Returns information about the bandwidth rate limit schedule of a gateway. By default, gateways do not have bandwidth rate limit schedules, which means no bandwidth rate limiting is in effect. This operation is supported only for volume, tape and S3 f",
        ),
    ),
    (
        "DescribeCache",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, DiskIds, CacheAllocatedInBytes, CacheUsedPercentage, CacheDirtyPercentage, CacheHitPercentage, CacheMissPercentage. Pinned purpose: Returns information about the cache of a gateway. This operation is only supported in the cached volume, tape, and file gateway types. The response includes disk IDs that are configured as cache, and it includes the amount of cache allocated and used",
        ),
    ),
    (
        "DescribeCacheReport",
        Decision::Deferred(
            "Required selectors CacheReportARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CacheReportInfo. Pinned purpose: Returns information about the specified cache report, including completion status and generation progress.",
        ),
    ),
    (
        "DescribeCachediSCSIVolumes",
        Decision::Deferred(
            "Required selectors VolumeARNs are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CachediSCSIVolumes. Pinned purpose: Returns a description of the gateway volumes specified in the request. This operation is only supported in the cached volume gateway types. The list of gateway volumes in the request must be from one gateway. In the response, Storage Gateway returns ",
        ),
    ),
    // Full modeled response contains authentication/secret material . Pinned purpose: Returns an array of Challenge-Handshake Authentication Protocol (CHAP) credentials information for a specified iSCSI target, one for each target-initiator pair. This operation is supported in the volume and tape gateway types.. Projection cannot authorize this call.
    (
        "DescribeChapCredentials",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "DescribeFileSystemAssociations",
        Decision::Deferred(
            "Required selectors FileSystemAssociationARNList are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FileSystemAssociationInfoList. Pinned purpose: Gets the file system association information. This operation is only supported for FSx File Gateways.",
        ),
    ),
    (
        "DescribeGatewayInformation",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, GatewayId, GatewayName, GatewayTimezone, GatewayState, GatewayNetworkInterfaces, GatewayType, NextUpdateAvailabilityDate, LastSoftwareUpdate, Ec2InstanceId, Ec2InstanceRegion, Tags, VPCEndpoint, CloudWatchLogGroupARN, HostEnvironment, EndpointType, SoftwareUpdatesEndDate, DeprecationDate, GatewayCapacity, SupportedGatewayCapacities, HostEnvironmentId, SoftwareVersion. Pinned purpose: Returns metadata about a gateway such as its name, network interfaces, time zone, status, and software version. To specify which gateway to describe, use the Amazon Resource Name (ARN) of the gateway in your request.",
        ),
    ),
    (
        "DescribeMaintenanceStartTime",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, HourOfDay, MinuteOfHour, DayOfWeek, DayOfMonth, Timezone, SoftwareUpdatePreferences. Pinned purpose: Returns your gateway's maintenance window schedule information, with values for monthly or weekly cadence, specific day and time to begin maintenance, and which types of updates to apply. Time values returned are for the gateway's time zone.",
        ),
    ),
    (
        "DescribeNFSFileShares",
        Decision::Deferred(
            "Required selectors FileShareARNList are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NFSFileShareInfoList. Pinned purpose: Gets a description for one or more Network File System (NFS) file shares from an S3 File Gateway. This operation is only supported for S3 File Gateways.",
        ),
    ),
    (
        "DescribeSMBFileShares",
        Decision::Deferred(
            "Required selectors FileShareARNList are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SMBFileShareInfoList. Pinned purpose: Gets a description for one or more Server Message Block (SMB) file shares from a S3 File Gateway. This operation is only supported for S3 File Gateways.",
        ),
    ),
    (
        "DescribeSMBSettings",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, DomainName, ActiveDirectoryStatus, SMBGuestPasswordSet, SMBSecurityStrategy, FileSharesVisible, SMBLocalGroups. Pinned purpose: Gets a description of a Server Message Block (SMB) file share settings from a file gateway. This operation is only supported for file gateways.",
        ),
    ),
    (
        "DescribeSnapshotSchedule",
        Decision::Deferred(
            "Required selectors VolumeARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members VolumeARN, StartAt, RecurrenceInHours, Description, Timezone, Tags. Pinned purpose: Describes the snapshot schedule for the specified gateway volume. The snapshot schedule information includes intervals at which snapshots are automatically initiated on the volume. This operation is only supported in the cached volume and stored volu",
        ),
    ),
    (
        "DescribeStorediSCSIVolumes",
        Decision::Deferred(
            "Required selectors VolumeARNs are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StorediSCSIVolumes. Pinned purpose: Returns the description of the gateway volumes specified in the request. The list of gateway volumes in the request must be from one gateway. In the response, Storage Gateway returns volume information sorted by volume ARNs. This operation is only su",
        ),
    ),
    (
        "DescribeTapeRecoveryPoints",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, TapeRecoveryPointInfos, Marker. Pinned purpose: Returns a list of virtual tape recovery points that are available for the specified tape gateway. A recovery point is a point-in-time view of a virtual tape at which all the data on the virtual tape is consistent. If your gateway crashes, virtual tap",
        ),
    ),
    (
        "DescribeTapes",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tapes, Marker. Pinned purpose: Returns a description of virtual tapes that correspond to the specified Amazon Resource Names (ARNs). If TapeARN is not specified, returns a description of the virtual tapes associated with the specified gateway. This operation is only supported for ",
        ),
    ),
    (
        "DescribeUploadBuffer",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, DiskIds, UploadBufferUsedInBytes, UploadBufferAllocatedInBytes. Pinned purpose: Returns information about the upload buffer of a gateway. This operation is supported for the stored volume, cached volume, and tape gateway types. The response includes disk IDs that are configured as upload buffer space, and it includes the amount ",
        ),
    ),
    (
        "DescribeVTLDevices",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, VTLDevices, Marker. Pinned purpose: Returns a description of virtual tape library (VTL) devices for the specified tape gateway. In the response, Storage Gateway returns VTL device information. This operation is only supported in the tape gateway type.",
        ),
    ),
    (
        "DescribeWorkingStorage",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, DiskIds, WorkingStorageUsedInBytes, WorkingStorageAllocatedInBytes. Pinned purpose: Returns information about the working storage of a gateway. This operation is only supported in the stored volumes gateway type. This operation is deprecated in cached volumes API version (20120630). Use DescribeUploadBuffer instead. Working storage ",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disconnects a volume from an iSCSI connection and then detaches the volume from the specified gateway. Detaching and attaching a volume enables you to recover your data from one gateway to a different gateway without creating a snapshot. It also makes it easier to move your volumes from an on-premises gateway to a gateway hosted on an Amazon EC2 instance. This operation is only supported in the vo
    ("DetachVolume", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disables a tape gateway when the gateway is no longer functioning. For example, if your gateway VM is damaged, you can disable the gateway so you can recover virtual tapes. Use this operation for a tape gateway that is not reachable or not functioning. This operation is only supported in the tape gateway type. After a gateway is disabled, it cannot be enabled.
    ("DisableGateway", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates an Amazon FSx file system from the specified gateway. After the disassociation process finishes, the gateway can no longer access the Amazon FSx file system. This operation is only supported in the FSx File Gateway type.
    (
        "DisassociateFileSystem",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a process that cleans the specified file share's cache of file entries that are failing upload to Amazon S3. This API operation reports success if the request is received with valid arguments, and there are no other cache clean operations currently in-progress for the specified file share. After a successful request, the cache clean operation occurs asynchronously and reports progress using
    (
        "EvictFilesFailingUpload",
        Decision::Denied(DenyReason::Mutating),
    ),
];
