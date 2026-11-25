use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("GetRoleCredentials", Decision::Denied(DenyReason::Secret)),
    (
        "ListAccountRoles",
        Decision::Deferred(
            "Required selectors accessToken, accountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, roleList. Pinned purpose: Lists all roles that are assigned to the user for a given AWS account.",
        ),
    ),
    (
        "ListAccounts",
        Decision::Deferred(
            "Required selectors accessToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, accountList. Pinned purpose: Lists all AWS accounts assigned to the user. These AWS accounts are assigned by the administrator of the account. For more information, see Assign User Access in the IAM Identity Center User Guide . This operation returns a paginated response.",
        ),
    ),
    ("Logout", Decision::Denied(DenyReason::Mutating)),
];
