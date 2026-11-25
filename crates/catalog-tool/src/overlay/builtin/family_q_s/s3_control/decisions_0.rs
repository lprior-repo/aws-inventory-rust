use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateAccessGrantsIdentityCenter",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAccessGrant", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAccessGrantsInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAccessGrantsLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAccessPoint", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAccessPointForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateBucket", Decision::Denied(DenyReason::Mutating)),
    ("CreateJob", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateMultiRegionAccessPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateStorageLensGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAccessGrant", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAccessGrantsInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccessGrantsInstanceResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccessGrantsLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAccessPoint", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAccessPointForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccessPointPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccessPointPolicyForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccessPointScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteBucket", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteBucketLifecycleConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteBucketPolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteBucketTagging",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteJobTagging", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteMultiRegionAccessPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeletePublicAccessBlock",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteStorageLensConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteStorageLensConfigurationTagging",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteStorageLensGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeJob",
        Decision::Deferred(
            "Required selectors AccountId, JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Job. Pinned purpose: Retrieves the configuration parameters and status for a Batch Operations job. For more information, see S3 Batch Operations in the Amazon S3 User Guide . Permissions To use the DescribeJob operation, you must have permission to perform the s3:Describ",
        ),
    ),
    (
        "DescribeMultiRegionAccessPointOperation",
        Decision::Deferred(
            "Required selectors AccountId, RequestTokenARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AsyncOperation. Pinned purpose: This operation is not supported by directory buckets. Retrieves the status of an asynchronous request to manage a Multi-Region Access Point. For more information about managing Multi-Region Access Points and how asynchronous requests work, see Using ",
        ),
    ),
    (
        "DissociateAccessGrantsIdentityCenter",
        Decision::Deferred(
            "Required selectors AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Dissociates the Amazon Web Services IAM Identity Center instance from the S3 Access Grants instance. Permissions You must have the s3:DissociateAccessGrantsIdentityCenter permission to use this operation. Additional Permissions You must have the sso:",
        ),
    ),
    (
        "GetAccessGrantsInstanceForPrefix",
        Decision::Deferred(
            "Required selectors AccountId, S3Prefix are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccessGrantsInstanceArn, AccessGrantsInstanceId. Pinned purpose: Retrieve the S3 Access Grants instance that contains a particular prefix. Permissions You must have the s3:GetAccessGrantsInstanceForPrefix permission for the caller account to use this operation. Additional Permissions The prefix owner account must ",
        ),
    ),
    (
        "GetAccessGrantsInstanceResourcePolicy",
        Decision::Deferred(
            "Required selectors AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy, Organization, CreatedAt. Pinned purpose: Returns the resource policy of the S3 Access Grants instance. Permissions You must have the s3:GetAccessGrantsInstanceResourcePolicy permission to use this operation.",
        ),
    ),
    (
        "GetAccessPoint",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, Bucket, NetworkOrigin, VpcConfiguration, PublicAccessBlockConfiguration, CreationDate, Alias, AccessPointArn, Endpoints, BucketAccountId, DataSourceId, DataSourceType. Pinned purpose: Returns configuration information about the specified access point. All Amazon S3 on Outposts REST API requests for this action require an additional parameter of x-amz-outpost-id to be passed with the request. In addition, you must use an S3 on Outp",
        ),
    ),
    (
        "GetAccessPointConfigurationForObjectLambda",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Configuration. Pinned purpose: This operation is not supported by directory buckets. Returns configuration for an Object Lambda Access Point. The following actions are related to GetAccessPointConfigurationForObjectLambda : PutAccessPointConfigurationForObjectLambda",
        ),
    ),
    (
        "GetAccessPointForObjectLambda",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, PublicAccessBlockConfiguration, CreationDate, Alias. Pinned purpose: This operation is not supported by directory buckets. Returns configuration information about the specified Object Lambda Access Point The following actions are related to GetAccessPointForObjectLambda : CreateAccessPointForObjectLambda DeleteAccessP",
        ),
    ),
    (
        "GetAccessPointPolicy",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: Returns the access point policy associated with the specified access point. The following actions are related to GetAccessPointPolicy : PutAccessPointPolicy DeleteAccessPointPolicy",
        ),
    ),
    (
        "GetAccessPointPolicyForObjectLambda",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: This operation is not supported by directory buckets. Returns the resource policy for an Object Lambda Access Point. The following actions are related to GetAccessPointPolicyForObjectLambda : DeleteAccessPointPolicyForObjectLambda PutAccessPointPolic",
        ),
    ),
    (
        "GetAccessPointPolicyStatus",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PolicyStatus. Pinned purpose: This operation is not supported by directory buckets. Indicates whether the specified access point currently has a policy that allows public access. For more information about public access through access points, see Managing Data Access with Amazon ",
        ),
    ),
    (
        "GetAccessPointPolicyStatusForObjectLambda",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PolicyStatus. Pinned purpose: This operation is not supported by directory buckets. Returns the status of the resource policy associated with an Object Lambda Access Point.",
        ),
    ),
    (
        "GetAccessPointScope",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Scope. Pinned purpose: Returns the access point scope for a directory bucket. To use this operation, you must have the permission to perform the s3express:GetAccessPointScope action. For information about REST API errors, see REST error responses .",
        ),
    ),
    (
        "GetBucket",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Bucket, PublicAccessBlockEnabled, CreationDate. Pinned purpose: Gets an Amazon S3 on Outposts bucket. For more information, see Using Amazon S3 on Outposts in the Amazon S3 User Guide . If you are using an identity other than the root user of the Amazon Web Services account that owns the Outposts bucket, the call",
        ),
    ),
    (
        "GetBucketLifecycleConfiguration",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Rules. Pinned purpose: This action gets an Amazon S3 on Outposts bucket's lifecycle configuration. To get an S3 bucket's lifecycle configuration, see GetBucketLifecycleConfiguration in the Amazon S3 API Reference . Returns the lifecycle configuration information set on the",
        ),
    ),
    (
        "GetBucketPolicy",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: This action gets a bucket policy for an Amazon S3 on Outposts bucket. To get a policy for an S3 bucket, see GetBucketPolicy in the Amazon S3 API Reference . Returns the policy of a specified Outposts bucket. For more information, see Using Amazon S3 ",
        ),
    ),
    (
        "GetBucketReplication",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ReplicationConfiguration. Pinned purpose: This operation gets an Amazon S3 on Outposts bucket's replication configuration. To get an S3 bucket's replication configuration, see GetBucketReplication in the Amazon S3 API Reference . Returns the replication configuration of an S3 on Outposts buc",
        ),
    ),
    (
        "GetBucketTagging",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TagSet. Pinned purpose: This action gets an Amazon S3 on Outposts bucket's tags. To get an S3 bucket tags, see GetBucketTagging in the Amazon S3 API Reference . Returns the tag set associated with the Outposts bucket. For more information, see Using Amazon S3 on Outposts in",
        ),
    ),
    (
        "GetBucketVersioning",
        Decision::Deferred(
            "Required selectors AccountId, Bucket are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, MFADelete. Pinned purpose: This operation returns the versioning state for S3 on Outposts buckets only. To return the versioning state for an S3 bucket, see GetBucketVersioning in the Amazon S3 API Reference . Returns the versioning state for an S3 on Outposts bucket. With S3 ",
        ),
    ),
    ("GetDataAccess", Decision::Denied(DenyReason::Secret)),
    (
        "GetJobTagging",
        Decision::Deferred(
            "Required selectors AccountId, JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tags. Pinned purpose: Returns the tags on an S3 Batch Operations job. Permissions To use the GetJobTagging operation, you must have permission to perform the s3:GetJobTagging action. For more information, see Controlling access and labeling jobs using tags in the Amazon S",
        ),
    ),
    (
        "GetMultiRegionAccessPoint",
        Decision::Deferred(
            "Required selectors AccountId, Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccessPoint. Pinned purpose: This operation is not supported by directory buckets. Returns configuration information about the specified Multi-Region Access Point. This action will always be routed to the US West (Oregon) Region. For more information about the restrictions aroun",
        ),
    ),
];
