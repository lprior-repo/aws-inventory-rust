use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("JoinDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "ListLocalDisks",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, Disks. Pinned purpose: Returns a list of the gateway's local disks. To specify which gateway to describe, you use the Amazon Resource Name (ARN) of the gateway in the body of the request. The request returns a list of all disks, specifying which are configured as working s",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Lists the tags that have been added to the specified resource. This operation is supported in storage gateways of all types.",
        ),
    ),
    (
        "ListVolumeInitiators",
        Decision::Deferred(
            "Required selectors VolumeARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Initiators. Pinned purpose: Lists iSCSI initiators that are connected to a volume. You can use this operation to determine whether a volume is being used or not. This operation is only supported in the cached volume and stored volume gateway types.",
        ),
    ),
    (
        "ListVolumeRecoveryPoints",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN, VolumeRecoveryPointInfos. Pinned purpose: Lists the recovery points for a specified gateway. This operation is only supported in the cached volume gateway type. Each cache volume has one recovery point. A volume recovery point is a point in time at which all data of the volume is consistent ",
        ),
    ),
    ("NotifyWhenUploaded", Decision::Denied(DenyReason::Mutating)),
    (
        "RefreshCache",
        Decision::Deferred(
            "Required selectors FileShareARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FileShareARN, NotificationId. Pinned purpose: Refreshes the cached inventory of objects for the specified file share. This operation finds objects in the Amazon S3 bucket that were added, removed, or replaced since the gateway last listed the bucket's contents and cached the results. This operat",
        ),
    ),
    (
        "RemoveTagsFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ResetCache", Decision::Denied(DenyReason::Mutating)),
    ("RetrieveTapeArchive", Decision::Denied(DenyReason::Content)),
    (
        "RetrieveTapeRecoveryPoint",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "SetLocalConsolePassword",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetSMBGuestPassword",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ShutdownGateway",
        Decision::Deferred(
            "Required selectors GatewayARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GatewayARN. Pinned purpose: Shuts down a Tape Gateway or Volume Gateway. To specify which gateway to shut down, use the Amazon Resource Name (ARN) of the gateway in the body of your request. This API action cannot be used to shut down S3 File Gateway or FSx File Gateway. The op",
        ),
    ),
    (
        "StartAvailabilityMonitorTest",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartCacheReport", Decision::Denied(DenyReason::Mutating)),
    ("StartGateway", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAutomaticTapeCreationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateBandwidthRateLimit",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateBandwidthRateLimitSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateChapCredentials",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "UpdateFileSystemAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateGatewayInformation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateGatewaySoftwareNow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMaintenanceStartTime",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateNFSFileShare", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSMBFileShare", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateSMBFileShareVisibility",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSMBLocalGroups",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSMBSecurityStrategy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateVTLDeviceType",
        Decision::Denied(DenyReason::Mutating),
    ),
];
