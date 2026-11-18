// Pinned operation purposes: .local/aws-models/models/supportauthz/service/2026-06-30/supportauthz-2026-06-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a support permit that authorizes an AWS support operator to perform specified actions on specified resources. The permit is cryptographically signed using a customer-managed AWS KMS key (ECC_NIST_P384, SIGN_VERIFY) to ensure non-repudiation.
    (
        "CreateSupportPermit",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a support permit, revoking the authorization previously granted to the AWS support operator.
    (
        "DeleteSupportPermit",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAction",
        Decision::Deferred(
            "Required selectors action are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members action, service, description. Pinned purpose: Retrieves the description of a specific support action.",
        ),
    ),
    (
        "GetSupportPermit",
        Decision::Deferred(
            "Required selectors supportPermitIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members name, arn, description, permit, status, signingKeyInfo, createdAt, supportCaseDisplayId, tags. Pinned purpose: Retrieves the details of a support permit by its ARN or name.",
        ),
    ),
    (
        "ListActions",
        Decision::Deferred(
            "Required selectors service are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members actionSummaries, nextToken. Pinned purpose: Lists available support actions for a specified AWS service. Use pagination to ensure that the operation returns quickly and successfully.",
        ),
    ),
    // Response permit is a signed support access permit/credential, not resource configuration. Never acquire or persist support authorization material for inventory.
    (
        "ListSupportPermitRequests",
        Decision::Denied(DenyReason::Secret),
    ),
    // Response includes support access authorization permits, not resource configuration; authorization material is secret-bearing.
    ("ListSupportPermits", Decision::Denied(DenyReason::Secret)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags associated with a support permit resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Rejects a permit request from an AWS support operator. The operator cannot proceed with the requested action.
    (
        "RejectSupportPermitRequest",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or overwrites one or more tags for a support permit resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from a support permit resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
