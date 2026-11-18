// Pinned operation purposes: .local/aws-models/models/s3-control/service/2018-08-20/s3-control-2018-08-20.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associate your S3 Access Grants instance with an Amazon Web Services IAM Identity Center instance. Use this action if you want to create access grants for users or groups from your corporate identity directory. First, you must add your corporate identity directory to Amazon Web Services IAM Identity Center. Then, you can associate this IAM Identity Center instance with your S3 Access Grants instan
    (
        "AssociateAccessGrantsIdentityCenter",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an access grant that gives a grantee access to your S3 data. The grantee can be an IAM user or role or a directory user, or group. Before you can create a grant, you must have an S3 Access Grants instance in the same Region as the S3 data. You can create an S3 Access Grants instance using the CreateAccessGrantsInstance . You must also have registered at least one S3 data location in your S
    ("CreateAccessGrant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an S3 Access Grants instance, which serves as a logical grouping for access grants. You can create one S3 Access Grants instance per Region per account. Permissions You must have the s3:CreateAccessGrantsInstance permission to use this operation. Additional Permissions To associate an IAM Identity Center instance with your S3 Access Grants instance, you must also have the sso:DescribeInsta
    (
        "CreateAccessGrantsInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: The S3 data location that you would like to register in your S3 Access Grants instance. Your S3 data must be in the same Region as your S3 Access Grants instance. The location can be one of the following: The default S3 location s3:// A bucket - S3:// A bucket and prefix - S3:// / When you register a location, you must include the IAM role that has permission to manage the S3 location that you are
    (
        "CreateAccessGrantsLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an access point and associates it to a specified bucket. For more information, see Managing access to shared datasets with access points or Managing access to shared datasets in directory buckets with access points in the Amazon S3 User Guide . To create an access point and attach it to a volume on an Amazon FSx file system, see CreateAndAttachS3AccessPoint in the Amazon FSx API Reference
    ("CreateAccessPoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Creates an Object Lambda Access Point. For more information, see Transforming objects with Object Lambda Access Points in the Amazon S3 User Guide . The following actions are related to CreateAccessPointForObjectLambda : DeleteAccessPointForObjectLambda GetAccessPointForObjectLambda ListAccessPointsForObjectLambda
    (
        "CreateAccessPointForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This action creates an Amazon S3 on Outposts bucket. To create an S3 bucket, see Create Bucket in the Amazon S3 API Reference . Creates a new Outposts bucket. By creating the bucket, you become the bucket owner. To create an Outposts bucket, you must have S3 on Outposts. For more information, see Using Amazon S3 on Outposts in Amazon S3 User Guide . Not every string is an acceptable bucket name. F
    ("CreateBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation creates an S3 Batch Operations job. You can use S3 Batch Operations to perform large-scale batch actions on Amazon S3 objects. Batch Operations can run a single action on lists of Amazon S3 objects that you specify. For more information, see S3 Batch Operations in the Amazon S3 User Guide . Permissions For information about permissions required to use the Batch Operations, see Grant
    ("CreateJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Creates a Multi-Region Access Point and associates it with the specified buckets. For more information about creating Multi-Region Access Points, see Creating Multi-Region Access Points in the Amazon S3 User Guide . This action will always be routed to the US West (Oregon) Region. For more information about the restrictions around working with
    (
        "CreateMultiRegionAccessPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new S3 Storage Lens group and associates it with the specified Amazon Web Services account ID. An S3 Storage Lens group is a custom grouping of objects based on prefix, suffix, object tags, object size, object age, or a combination of these filters. For each Storage Lens group that you’ve created, you can also optionally add Amazon Web Services resource tags. For more information about S
    (
        "CreateStorageLensGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the access grant from the S3 Access Grants instance. You cannot undo an access grant deletion and the grantee will no longer have access to the S3 data. Permissions You must have the s3:DeleteAccessGrant permission to use this operation.
    ("DeleteAccessGrant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes your S3 Access Grants instance. You must first delete the access grants and locations before S3 Access Grants can delete the instance. See DeleteAccessGrant and DeleteAccessGrantsLocation . If you have associated an IAM Identity Center instance with your S3 Access Grants instance, you must first dissassociate the Identity Center instance from the S3 Access Grants instance before you can de
    (
        "DeleteAccessGrantsInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the resource policy of the S3 Access Grants instance. The resource policy is used to manage cross-account access to your S3 Access Grants instance. By deleting the resource policy, you delete any cross-account permissions to your S3 Access Grants instance. Permissions You must have the s3:DeleteAccessGrantsInstanceResourcePolicy permission to use this operation.
    (
        "DeleteAccessGrantsInstanceResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deregisters a location from your S3 Access Grants instance. You can only delete a location registration from an S3 Access Grants instance if there are no grants associated with this location. See Delete a grant for information on how to delete grants. You need to have at least one registered location in your S3 Access Grants instance in order to create access grants. Permissions You must have the
    (
        "DeleteAccessGrantsLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified access point. All Amazon S3 on Outposts REST API requests for this action require an additional parameter of x-amz-outpost-id to be passed with the request. In addition, you must use an S3 on Outposts endpoint hostname prefix instead of s3-control . For an example of the request syntax for Amazon S3 on Outposts that uses the S3 on Outposts endpoint hostname prefix and the x-a
    ("DeleteAccessPoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Deletes the specified Object Lambda Access Point. The following actions are related to DeleteAccessPointForObjectLambda : CreateAccessPointForObjectLambda GetAccessPointForObjectLambda ListAccessPointsForObjectLambda
    (
        "DeleteAccessPointForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the access point policy for the specified access point. All Amazon S3 on Outposts REST API requests for this action require an additional parameter of x-amz-outpost-id to be passed with the request. In addition, you must use an S3 on Outposts endpoint hostname prefix instead of s3-control . For an example of the request syntax for Amazon S3 on Outposts that uses the S3 on Outposts endpoint
    (
        "DeleteAccessPointPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Removes the resource policy for an Object Lambda Access Point. The following actions are related to DeleteAccessPointPolicyForObjectLambda : GetAccessPointPolicyForObjectLambda PutAccessPointPolicyForObjectLambda
    (
        "DeleteAccessPointPolicyForObjectLambda",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing access point scope for a directory bucket. When you delete the scope of an access point, all prefixes and permissions are deleted. To use this operation, you must have the permission to perform the s3express:DeleteAccessPointScope action. For information about REST API errors, see REST error responses .
    (
        "DeleteAccessPointScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This action deletes an Amazon S3 on Outposts bucket. To delete an S3 bucket, see DeleteBucket in the Amazon S3 API Reference . Deletes the Amazon S3 on Outposts bucket. All objects (including all object versions and delete markers) in the bucket must be deleted before the bucket itself can be deleted. For more information, see Using Amazon S3 on Outposts in Amazon S3 User Guide . All Amazon S3 on
    ("DeleteBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This action deletes an Amazon S3 on Outposts bucket's lifecycle configuration. To delete an S3 bucket's lifecycle configuration, see DeleteBucketLifecycle in the Amazon S3 API Reference . Deletes the lifecycle configuration from the specified Outposts bucket. Amazon S3 on Outposts removes all the lifecycle configuration rules in the lifecycle subresource associated with the bucket. Your objects ne
    (
        "DeleteBucketLifecycleConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This action deletes an Amazon S3 on Outposts bucket policy. To delete an S3 bucket policy, see DeleteBucketPolicy in the Amazon S3 API Reference . This implementation of the DELETE action uses the policy subresource to delete the policy of a specified Amazon S3 on Outposts bucket. If you are using an identity other than the root user of the Amazon Web Services account that owns the bucket, the cal
    ("DeleteBucketPolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation deletes an Amazon S3 on Outposts bucket's replication configuration. To delete an S3 bucket's replication configuration, see DeleteBucketReplication in the Amazon S3 API Reference . Deletes the replication configuration from the specified S3 on Outposts bucket. To use this operation, you must have permissions to perform the s3-outposts:PutReplicationConfiguration action. The Outpost
    (
        "DeleteBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This action deletes an Amazon S3 on Outposts bucket's tags. To delete an S3 bucket tags, see DeleteBucketTagging in the Amazon S3 API Reference . Deletes the tags from the Outposts bucket. For more information, see Using Amazon S3 on Outposts in Amazon S3 User Guide . To use this action, you must have permission to perform the PutBucketTagging action. By default, the bucket owner has this permissi
    (
        "DeleteBucketTagging",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the entire tag set from the specified S3 Batch Operations job. Permissions To use the DeleteJobTagging operation, you must have permission to perform the s3:DeleteJobTagging action. For more information, see Controlling access and labeling jobs using tags in the Amazon S3 User Guide . Related actions include: CreateJob GetJobTagging PutJobTagging
    ("DeleteJobTagging", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Deletes a Multi-Region Access Point. This action does not delete the buckets associated with the Multi-Region Access Point, only the Multi-Region Access Point itself. This action will always be routed to the US West (Oregon) Region. For more information about the restrictions around working with Multi-Region Access Points, see Multi-Region Acce
    (
        "DeleteMultiRegionAccessPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Removes the PublicAccessBlock configuration for an Amazon Web Services account. This operation might be restricted when the account is managed by organization-level Block Public Access policies. You’ll get an Access Denied (403) error when the account is managed by organization-level Block Public Access policies. Organization-level policies ove
    (
        "DeletePublicAccessBlock",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Deletes the Amazon S3 Storage Lens configuration. For more information about S3 Storage Lens, see Assessing your storage activity and usage with Amazon S3 Storage Lens in the Amazon S3 User Guide . To use this action, you must have permission to perform the s3:DeleteStorageLensConfiguration action. For more information, see Setting permissions
    (
        "DeleteStorageLensConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation is not supported by directory buckets. Deletes the Amazon S3 Storage Lens configuration tags. For more information about S3 Storage Lens, see Assessing your storage activity and usage with Amazon S3 Storage Lens in the Amazon S3 User Guide . To use this action, you must have permission to perform the s3:DeleteStorageLensConfigurationTagging action. For more information, see Setting
    (
        "DeleteStorageLensConfigurationTagging",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing S3 Storage Lens group. To use this operation, you must have the permission to perform the s3:DeleteStorageLensGroup action. For more information about the required Storage Lens Groups permissions, see Setting account permissions to use S3 Storage Lens groups . For information about Storage Lens groups errors, see List of Amazon S3 Storage Lens error codes .
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
    // Full modeled response contains authentication/secret material Credentials.SecretAccessKey, Credentials.SessionToken. Pinned purpose: Returns a temporary access credential from S3 Access Grants to the grantee or client application. The temporary credential is an Amazon Web Services STS token that grants them access to the S3 data. By default, S3 Access Grants searches for a matching grant in the instance of the Amazon Web Services. Projection cannot authorize this call.
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
