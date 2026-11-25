use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AddProfilePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CancelSigningProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeSigningJob",
        Decision::Deferred(
            "Required selectors jobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members jobId, source, signingMaterial, platformId, platformDisplayName, profileName, profileVersion, overrides, signingParameters, createdAt, completedAt, signatureExpiresAt, requestedBy, status, statusReason, revocationRecord, signedObject, jobOwner, jobInvoker. Pinned purpose: Returns information about a specific code signing job. You specify the job by using the jobId value that is returned by the StartSigningJob operation.",
        ),
    ),
    (
        "GetRevocationStatus",
        Decision::Deferred(
            "Required selectors signatureTimestamp, platformId, profileVersionArn, jobArn, certificateHashes are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members revokedEntities. Pinned purpose: Retrieves the revocation status of one or more of the signing profile, signing job, and signing certificate.",
        ),
    ),
    (
        "GetSigningPlatform",
        Decision::Deferred(
            "Required selectors platformId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members platformId, displayName, partner, target, category, signingConfiguration, signingImageFormat, maxSizeInMB, revocationSupported. Pinned purpose: Returns information on a specific signing platform.",
        ),
    ),
    (
        "GetSigningProfile",
        Decision::Deferred(
            "Required selectors profileName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members profileName, profileVersion, profileVersionArn, revocationRecord, signingMaterial, platformId, platformDisplayName, signatureValidityPeriod, overrides, signingParameters, status, statusReason, arn, tags. Pinned purpose: Returns information on a specific signing profile.",
        ),
    ),
    (
        "ListProfilePermissions",
        Decision::Deferred(
            "Required selectors profileName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members revisionId, policySizeBytes, permissions, nextToken. Pinned purpose: Lists the cross-account permissions associated with a signing profile.",
        ),
    ),
    (
        "ListSigningPlatforms",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns a list of the tags associated with a signing profile resource.",
        ),
    ),
    ("PutSigningProfile", Decision::Denied(DenyReason::Mutating)),
    (
        "RemoveProfilePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("RevokeSignature", Decision::Denied(DenyReason::Mutating)),
    (
        "RevokeSigningProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SignPayload",
        Decision::Deferred(
            "Required selectors profileName, payload, payloadFormat are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members jobId, jobOwner, metadata, signature. Pinned purpose: Signs a binary payload and returns a signature envelope.",
        ),
    ),
    ("StartSigningJob", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
