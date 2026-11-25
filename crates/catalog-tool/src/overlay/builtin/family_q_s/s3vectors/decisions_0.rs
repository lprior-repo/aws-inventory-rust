use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateIndex", Decision::Denied(DenyReason::Mutating)),
    ("CreateVectorBucket", Decision::Denied(DenyReason::Mutating)),
    ("DeleteIndex", Decision::Denied(DenyReason::Mutating)),
    ("DeleteVectorBucket", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteVectorBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("ListVectors", Decision::Denied(DenyReason::Content)),
    (
        "PutVectorBucketDefaultIndexMode",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutVectorBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutVectors", Decision::Denied(DenyReason::Mutating)),
    (
        "QueryVectors",
        Decision::Deferred(
            "Required selectors topK, queryVector are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members vectors, distanceMetric, nextToken. Pinned purpose: Performs an approximate nearest neighbor search query in a vector index using a query vector. By default, it returns the keys of approximate nearest neighbors. You can optionally include the computed distance (between the query vector and each vector",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateIndexMode", Decision::Denied(DenyReason::Mutating)),
];
