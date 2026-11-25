use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "GetMultiRegionAccessPointPolicy",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: This operation is not supported by directory buckets. Returns the access control policy of the specified Multi-Region Access Point. This action will always be routed to the US West (Oregon) Region. For more information about the restrictions around w",
        ),
    ),
    (
        "GetMultiRegionAccessPointPolicyStatus",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Established. Pinned purpose: This operation is not supported by directory buckets. Indicates whether the specified Multi-Region Access Point has an access control policy that allows public access. This action will always be routed to the US West (Oregon) Region. For more informa",
        ),
    ),
    (
        "GetMultiRegionAccessPointRoutes",
        Decision::Deferred(
            "Required selectors AccountId, Mrap are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Mrap, Routes. Pinned purpose: This operation is not supported by directory buckets. Returns the routing configuration for a Multi-Region Access Point, indicating which Regions are active or passive. To obtain routing control changes and failover requests, use the Amazon S3 failov",
        ),
    ),
    (
        "GetPublicAccessBlock",
        Decision::Deferred(
            "Required selectors AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PublicAccessBlockConfiguration. Pinned purpose: This operation is not supported by directory buckets. Retrieves the PublicAccessBlock configuration for an Amazon Web Services account. This operation returns the effective account-level configuration, which may inherit from organization-level polici",
        ),
    ),
    (
        "GetStorageLensConfiguration",
        Decision::Deferred(
            "Required selectors ConfigId, AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StorageLensConfiguration. Pinned purpose: This operation is not supported by directory buckets. Gets the Amazon S3 Storage Lens configuration. For more information, see Assessing your storage activity and usage with Amazon S3 Storage Lens in the Amazon S3 User Guide . For a complete list of ",
        ),
    ),
    (
        "GetStorageLensConfigurationTagging",
        Decision::Deferred(
            "Required selectors ConfigId, AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tags. Pinned purpose: This operation is not supported by directory buckets. Gets the tags of Amazon S3 Storage Lens configuration. For more information about S3 Storage Lens, see Assessing your storage activity and usage with Amazon S3 Storage Lens in the Amazon S3 User G",
        ),
    ),
    (
        "ListAccessPointsForDirectoryBuckets",
        Decision::Deferred(
            "Required selectors AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccessPointList, NextToken. Pinned purpose: Returns a list of the access points that are owned by the Amazon Web Services account and that are associated with the specified directory bucket. To list access points for general purpose buckets, see ListAccesspoints . To use this operation, you mu",
        ),
    ),
    (
        "ListCallerAccessGrants",
        Decision::Deferred(
            "Required selectors AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, CallerAccessGrantsList. Pinned purpose: Use this API to list the access grants that grant the caller access to Amazon S3 data through S3 Access Grants. The caller (grantee) can be an Identity and Access Management (IAM) identity or Amazon Web Services Identity Center corporate directory id",
        ),
    ),
    (
        "ListMultiRegionAccessPoints",
        Decision::Deferred(
            "Model hostPrefix {AccountId}. and hostLabel are supported, but the documented control-plane request must route/sign in us-west-2. Current Account/HomeRegion scope uses caller home and Region fans out; no fixed service routing contract exists for this global resource.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: AccountId, ResourceArn. Pinned purpose: This operation allows you to list all of the tags for a specified resource. Each tag is a label consisting of a key and value. Tags can help you organize, track costs for, and control access to resources. This operation is only supported for the following Amazon S3 resources: General purpose buckets",
        ),
    ),
    (
        "PutAccessGrantsInstanceResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccessPointConfigurationForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccessPointPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccessPointPolicyForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccessPointScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutBucketLifecycleConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutBucketPolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "PutBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutBucketTagging", Decision::Denied(DenyReason::Mutating)),
    (
        "PutBucketVersioning",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutJobTagging", Decision::Denied(DenyReason::Mutating)),
    (
        "PutMultiRegionAccessPointPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutPublicAccessBlock",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutStorageLensConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutStorageLensConfigurationTagging",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SubmitMultiRegionAccessPointRoutes",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAccessGrantsLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateJobPriority", Decision::Denied(DenyReason::Mutating)),
    ("UpdateJobStatus", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateStorageLensGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
];
