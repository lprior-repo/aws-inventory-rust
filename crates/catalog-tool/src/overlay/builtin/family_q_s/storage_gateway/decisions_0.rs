use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ActivateGateway",
        Decision::Deferred(
            "Required selectors ActivationKey, GatewayName, GatewayTimezone, GatewayRegion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN. Pinned purpose: Activates the gateway you previously deployed on your host. In the activation process, you specify information such as the Amazon Web Services Region that you want to use for storing snapshots or tapes, the time zone for scheduled snapshots the gatew",
        ),
    ),
    ("AddCache", Decision::Denied(DenyReason::Mutating)),
    ("AddTagsToResource", Decision::Denied(DenyReason::Mutating)),
    ("AddUploadBuffer", Decision::Denied(DenyReason::Mutating)),
    ("AddWorkingStorage", Decision::Denied(DenyReason::Mutating)),
    (
        "AssignTapePool",
        Decision::Deferred(
            "Required selectors TapeARN, PoolId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TapeARN. Pinned purpose: Assigns a tape to a tape pool for archiving. The tape assigned to a pool is archived in the S3 storage class that is associated with the pool. When you use your backup application to eject the tape, the tape is archived directly into the S3 storage c",
        ),
    ),
    (
        "AssociateFileSystem",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("AttachVolume", Decision::Denied(DenyReason::Mutating)),
    ("CancelArchival", Decision::Denied(DenyReason::Mutating)),
    ("CancelCacheReport", Decision::Denied(DenyReason::Mutating)),
    ("CancelRetrieval", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateCachediSCSIVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateNFSFileShare", Decision::Denied(DenyReason::Mutating)),
    ("CreateSMBFileShare", Decision::Denied(DenyReason::Mutating)),
    ("CreateSnapshot", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateSnapshotFromVolumeRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateStorediSCSIVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTapePool", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTapeWithBarcode",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTapes", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAutomaticTapeCreationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteBandwidthRateLimit",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteCacheReport", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteChapCredentials",
        Decision::Denied(DenyReason::Secret),
    ),
    ("DeleteFileShare", Decision::Denied(DenyReason::Mutating)),
    ("DeleteGateway", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTape", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTapeArchive", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTapePool", Decision::Denied(DenyReason::Mutating)),
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
    ("DetachVolume", Decision::Denied(DenyReason::Mutating)),
    ("DisableGateway", Decision::Denied(DenyReason::Mutating)),
    (
        "DisassociateFileSystem",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EvictFilesFailingUpload",
        Decision::Denied(DenyReason::Mutating),
    ),
];
