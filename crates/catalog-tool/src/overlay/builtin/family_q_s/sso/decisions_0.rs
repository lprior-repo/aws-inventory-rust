// Pinned operation purposes: .local/aws-models/models/sso/service/2019-06-10/sso-2019-06-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Full modeled response contains authentication/secret material roleCredentials.secretAccessKey, roleCredentials.sessionToken. Pinned purpose: Returns the STS short-term credentials for a given role name that is assigned to the user.. Projection cannot authorize this call.
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
    // Changes service state rather than reading inventory. Pinned purpose: Removes the locally stored SSO tokens from the client-side cache and sends an API call to the IAM Identity Center service to invalidate the corresponding server-side IAM Identity Center sign in session. If a user uses IAM Identity Center to access the AWS CLI, the user’s IAM Identity Center sign in session is used to obtain an IAM session, as specified in the corresponding IAM Identity Center perm
    ("Logout", Decision::Denied(DenyReason::Mutating)),
];
