// Pinned operation purposes: .local/aws-models/models/s3files/service/2025-05-05/s3files-2025-05-05.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates an S3 File System Access Point for application-specific access with POSIX user identity and root directory enforcement. Access points provide a way to manage access to shared datasets in multi-tenant scenarios.
    ("CreateAccessPoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an S3 File System resource scoped to a bucket or prefix within a bucket, enabling file system access to S3 data. To create a file system, you need an S3 bucket and an IAM role that grants the service permission to access the bucket.
    ("CreateFileSystem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a mount target resource as an endpoint for mounting the S3 File System from compute resources in a specific Availability Zone and VPC. Mount targets provide network access to the file system.
    ("CreateMountTarget", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an S3 File System Access Point. This operation is irreversible.
    ("DeleteAccessPoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an S3 File System. You can optionally force deletion of a file system that has pending export data.
    ("DeleteFileSystem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the IAM resource policy of an S3 File System.
    (
        "DeleteFileSystemPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified mount target. This operation is irreversible.
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
    // Changes service state rather than reading inventory. Pinned purpose: Creates or replaces the IAM resource policy for an S3 File System to control access permissions.
    (
        "PutFileSystemPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates the synchronization configuration for the specified S3 File System, including import data rules and expiration data rules.
    (
        "PutSynchronizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates tags for S3 Files resources using standard Amazon Web Services tagging APIs.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from S3 Files resources.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the mount target resource, specifically security group configurations.
    ("UpdateMountTarget", Decision::Denied(DenyReason::Mutating)),
];
