use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateBearerToken", Decision::Denied(DenyReason::Secret)),
    ("CreateOAuth2Token", Decision::Denied(DenyReason::Secret)),
    (
        "CreateOAuth2TokenWithIAM",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "CreateOAuth2TokenWithTokenExchange",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "DeleteConsoleAuthorizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourcePermissionStatement",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetConsoleAuthorizationConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members targetId, scope, consoleAuthorizationEnabled. Pinned request/ownership contract: Get console authorization configuration with automatic scope detection",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members signinResourceBasedPolicy. Pinned request/ownership contract: Retrieve the account's consolidated SignIn resource-based policy",
        ),
    ),
    (
        "IntrospectOAuth2TokenWithIAM",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ListResourcePermissionStatements",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members permissionStatements, nextToken. Pinned request/ownership contract: Retrieve all permission statements in the account's SignIn resource-based policy",
        ),
    ),
    (
        "PutConsoleAuthorizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutResourcePermissionStatement",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RevokeOAuth2TokenWithIAM",
        Decision::Denied(DenyReason::Secret),
    ),
];
