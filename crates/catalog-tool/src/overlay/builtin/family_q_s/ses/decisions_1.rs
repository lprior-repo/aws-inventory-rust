use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "UpdateConfigurationSetReputationMetricsEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationSetSendingEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateReceiptRule", Decision::Denied(DenyReason::Mutating)),
    ("UpdateTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "VerifyDomainDkim",
        Decision::Deferred(
            "Required selectors Domain are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DkimTokens. Pinned purpose: Returns a set of DKIM tokens for a domain identity. When you execute the VerifyDomainDkim operation, the domain that you specify is added to the list of identities that are associated with your account. This is true even if you haven't already associ",
        ),
    ),
    (
        "VerifyDomainIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "VerifyEmailAddress",
        Decision::Deferred(
            "Required selectors EmailAddress are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Deprecated. Use the VerifyEmailIdentity operation to verify a new email address.",
        ),
    ),
    (
        "VerifyEmailIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
];
