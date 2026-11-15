// Pinned operation purposes: .local/aws-models/models/simpledbv2/service/2025-09-26/simpledbv2-2025-09-26.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("GetExport", Decision::Deferred("Required selectors exportArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members exportArn, clientToken, exportStatus, domainName, requestedAt, s3Bucket, s3KeyPrefix, s3SseAlgorithm, s3SseKmsKeyId, s3BucketOwner, failureCode, failureMessage, exportManifest, itemsCount, exportDataCutoffTime. Pinned purpose: Returns information for an existing domain export.")),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates the export of a SimpleDB domain to an S3 bucket.
    ("StartDomainExport", Decision::Denied(DenyReason::Mutating)),
];
