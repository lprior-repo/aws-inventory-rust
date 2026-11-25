use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateSupportPermit",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    (
        "ListSupportPermitRequests",
        Decision::Denied(DenyReason::Secret),
    ),
    ("ListSupportPermits", Decision::Denied(DenyReason::Secret)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags associated with a support permit resource.",
        ),
    ),
    (
        "RejectSupportPermitRequest",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
