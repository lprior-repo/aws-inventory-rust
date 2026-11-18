// Pinned operation purposes: .local/aws-models/models/s3vectors/service/2025-07-15/s3vectors-2025-07-15.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a vector index within a vector bucket. To specify the vector bucket, you must use either the vector bucket name or the vector bucket Amazon Resource Name (ARN). Permissions You must have the s3vectors:CreateIndex permission to use this operation. You must have the s3vectors:TagResource permission in addition to s3vectors:CreateIndex permission to create a vector index with tags.
    ("CreateIndex", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a vector bucket in the Amazon Web Services Region that you want your bucket to be in. Permissions You must have the s3vectors:CreateVectorBucket permission to use this operation. You must have the s3vectors:TagResource permission in addition to s3vectors:CreateVectorBucket permission to create a vector bucket with tags.
    ("CreateVectorBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a vector index. To specify the vector index, you can either use both the vector bucket name and vector index name, or use the vector index Amazon Resource Name (ARN). Permissions You must have the s3vectors:DeleteIndex permission to use this operation.
    ("DeleteIndex", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a vector bucket. All vector indexes in the vector bucket must be deleted before the vector bucket can be deleted. To perform this operation, you must use either the vector bucket name or the vector bucket Amazon Resource Name (ARN). Permissions You must have the s3vectors:DeleteVectorBucket permission to use this operation.
    ("DeleteVectorBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a vector bucket policy. To specify the bucket, you must use either the vector bucket name or the vector bucket Amazon Resource Name (ARN). Permissions You must have the s3vectors:DeleteVectorBucketPolicy permission to use this operation.
    (
        "DeleteVectorBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes one or more vectors in a vector index. To specify the vector index, you can either use both the vector bucket name and vector index name, or use the vector index Amazon Resource Name (ARN). Permissions You must have the s3vectors:DeleteVectors permission to use this operation.
    ("DeleteVectors", Decision::Denied(DenyReason::Mutating)),
    (
        "GetIndex",
        Decision::Deferred(
            "Requires either indexArn or vectorBucketName/vectorBucketArn plus indexName, a semantic union not encoded as required fields. ListIndexes provides complete per-vector-bucket metadata.",
        ),
    ),
    (
        "GetVectorBucket",
        Decision::Deferred(
            "Requires either vectorBucketName or vectorBucketArn despite optional model members. ListVectorBuckets is the complete root.",
        ),
    ),
    (
        "GetVectorBucketPolicy",
        Decision::Deferred(
            "Caller must select vectorBucketName or vectorBucketArn. The policy JSON is authorization content rather than a distinct resource root.",
        ),
    ),
    (
        "GetVectors",
        Decision::Deferred(
            "Required selectors keys are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members vectors. Pinned purpose: Returns vector attributes. To specify the vector index, you can either use both the vector bucket name and the vector index name, or use the vector index Amazon Resource Name (ARN). Permissions You must have the s3vectors:GetVectors permission to use",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists all of the tags applied to a specified Amazon S3 Vectors resource. Each tag is a label consisting of a key and value pair. Tags can help you organize, track costs for, and control access to resources. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . ",
        ),
    ),
    // Reads customer vector keys/data/metadata. Data-plane content is not permitted even if the caller could omit vectors from a projection.
    ("ListVectors", Decision::Denied(DenyReason::Content)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the default index mode for a vector bucket. The updated default applies to vector indexes that you create after the request succeeds. The operation doesn't change existing vector indexes. To specify the vector bucket, you must use either the vector bucket name or the vector bucket Amazon Resource Name (ARN). Permissions You must have the s3vectors:PutVectorBucketDefaultIndexMode permission
    (
        "PutVectorBucketDefaultIndexMode",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a bucket policy for a vector bucket. To specify the bucket, you must use either the vector bucket name or the vector bucket Amazon Resource Name (ARN). Permissions You must have the s3vectors:PutVectorBucketPolicy permission to use this operation.
    (
        "PutVectorBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds one or more vectors to a vector index. To specify the vector index, you can either use both the vector bucket name and the vector index name, or use the vector index Amazon Resource Name (ARN). For more information about limits, see Limitations and restrictions in the Amazon S3 User Guide . When inserting vector data into your vector index, you must provide the vector data as float32 (32-bit
    ("PutVectors", Decision::Denied(DenyReason::Mutating)),
    (
        "QueryVectors",
        Decision::Deferred(
            "Required selectors topK, queryVector are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members vectors, distanceMetric, nextToken. Pinned purpose: Performs an approximate nearest neighbor search query in a vector index using a query vector. By default, it returns the keys of approximate nearest neighbors. You can optionally include the computed distance (between the query vector and each vector",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Applies one or more user-defined tags to an Amazon S3 Vectors resource or updates existing tags. Each tag is a label consisting of a key and value pair. Tags can help you organize, track costs for, and control access to your resources. You can add up to 50 tags for each resource. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . Permissions For vector buc
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified user-defined tags from an Amazon S3 Vectors resource. You can pass one or more tag keys. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . Permissions For vector buckets and vector indexes, you must have the s3vectors:UntagResource permission to use this operation.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the mode for an existing vector index. You can set the mode to ENHANCED for any vector index. You can set the mode to CLASSIC only for a vector index in a vector bucket created before September 30, 2026. This operation doesn't change the default index mode of the vector bucket or the mode of other vector indexes. Specify the vector index by using its Amazon Resource Name (ARN) or both the
    ("UpdateIndexMode", Decision::Denied(DenyReason::Mutating)),
];
