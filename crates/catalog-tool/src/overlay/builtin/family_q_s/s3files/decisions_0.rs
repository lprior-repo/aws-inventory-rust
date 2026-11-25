use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateAccessPoint", Decision::Denied(DenyReason::Mutating)),
    ("CreateFileSystem", Decision::Denied(DenyReason::Mutating)),
    ("CreateMountTarget", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAccessPoint", Decision::Denied(DenyReason::Mutating)),
    ("DeleteFileSystem", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFileSystemPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteMountTarget", Decision::Denied(DenyReason::Mutating)),
    (
        "GetAccessPoint",
        Decision::Deferred(
            "Required selectors accessPointId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members accessPointArn, accessPointId, clientToken, fileSystemId, status, ownerId, posixUser, rootDirectory, tags, name. Pinned purpose: Returns resource information for an S3 File System Access Point.",
        ),
    ),
    (
        "GetFileSystem",
        Decision::Deferred(
            "Required selectors fileSystemId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members creationTime, fileSystemArn, fileSystemId, bucket, prefix, clientToken, kmsKeyId, status, statusMessage, roleArn, ownerId, tags, name. Pinned purpose: Returns resource information for the specified S3 File System including status, configuration, and metadata.",
        ),
    ),
    (
        "GetFileSystemPolicy",
        Decision::Deferred(
            "Required selectors fileSystemId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members fileSystemId, policy. Pinned purpose: Returns the IAM resource policy of an S3 File System.",
        ),
    ),
    (
        "GetMountTarget",
        Decision::Deferred(
            "Required selectors mountTargetId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members availabilityZoneId, ownerId, mountTargetId, fileSystemId, subnetId, ipv4Address, ipv6Address, networkInterfaceId, vpcId, securityGroups, status, statusMessage. Pinned purpose: Returns detailed resource information for the specified mount target including network configuration.",
        ),
    ),
    (
        "GetSynchronizationConfiguration",
        Decision::Deferred(
            "Required selectors fileSystemId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members latestVersionNumber, importDataRules, expirationDataRules. Pinned purpose: Returns the synchronization configuration for the specified S3 File System, including import data rules and expiration data rules.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceId. Pinned purpose: Lists all tags for S3 Files resources.",
        ),
    ),
    (
        "PutFileSystemPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutSynchronizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateMountTarget", Decision::Denied(DenyReason::Mutating)),
];
