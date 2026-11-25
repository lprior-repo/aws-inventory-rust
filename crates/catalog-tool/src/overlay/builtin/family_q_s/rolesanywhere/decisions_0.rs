use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateProfile", Decision::Denied(DenyReason::Mutating)),
    ("CreateTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAttributeMapping",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteCrl", Decision::Denied(DenyReason::Mutating)),
    ("DeleteProfile", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    ("DisableCrl", Decision::Denied(DenyReason::Mutating)),
    ("DisableProfile", Decision::Denied(DenyReason::Mutating)),
    ("DisableTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    ("EnableCrl", Decision::Denied(DenyReason::Mutating)),
    ("EnableProfile", Decision::Denied(DenyReason::Mutating)),
    ("EnableTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    (
        "GetCrl",
        Decision::Deferred(
            "Required selectors crlId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members crl. Pinned purpose: Gets a certificate revocation list (CRL). Required permissions: rolesanywhere:GetCrl .",
        ),
    ),
    (
        "GetProfile",
        Decision::Deferred(
            "Required selectors profileId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members profile. Pinned purpose: Gets a profile. Required permissions: rolesanywhere:GetProfile .",
        ),
    ),
    (
        "GetSubject",
        Decision::Deferred(
            "Required selectors subjectId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members subject. Pinned purpose: Gets a subject , which associates a certificate identity with authentication attempts. The subject stores auditing information such as the status of the last authentication attempt, the certificate data used in the attempt, and the last time the asso",
        ),
    ),
    (
        "GetTrustAnchor",
        Decision::Deferred(
            "Required selectors trustAnchorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members trustAnchor. Pinned purpose: Gets a trust anchor. Required permissions: rolesanywhere:GetTrustAnchor .",
        ),
    ),
    ("ImportCrl", Decision::Denied(DenyReason::Mutating)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags attached to the resource. Required permissions: rolesanywhere:ListTagsForResource .",
        ),
    ),
    (
        "PutAttributeMapping",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutNotificationSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ResetNotificationSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCrl", Decision::Denied(DenyReason::Mutating)),
    ("UpdateProfile", Decision::Denied(DenyReason::Mutating)),
    ("UpdateTrustAnchor", Decision::Denied(DenyReason::Mutating)),
];
