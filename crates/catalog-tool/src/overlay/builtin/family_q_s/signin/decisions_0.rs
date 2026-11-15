// Pinned operation purposes: .local/aws-models/models/signin/service/2023-01-01/signin-2023-01-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Full modeled response contains authentication/secret material accessToken. Pinned purpose: Creates a bearer token that represents your IAM credentials and can be used to access a supported AWS service. By default, the token is valid for 12 hours.. Projection cannot authorize this call.
    ("CreateBearerToken", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material tokenOutput.accessToken, tokenOutput.accessToken.secretAccessKey, tokenOutput.accessToken.sessionToken, tokenOutput.refreshToken. Pinned purpose: Exchanges an authorization code or refresh token for an OAuth 2.0 access token and refresh token.. Projection cannot authorize this call.
    ("CreateOAuth2Token", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material accessToken. Pinned purpose: Exchanges client credentials for an OAuth 2.0 access token scoped to a resource, which you can use to access AWS services from applications.. Projection cannot authorize this call.
    ("CreateOAuth2TokenWithIAM", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material accessToken. Pinned purpose: Exchanges a supported token for an OAuth 2.0 access token scoped to a resource, which you can use to access AWS services from applications.. Projection cannot authorize this call.
    ("CreateOAuth2TokenWithTokenExchange", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete console authorization configuration with automatic scope detection
    ("DeleteConsoleAuthorizationConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Remove a permission statement from the account's SignIn resource-based policy
    ("DeleteResourcePermissionStatement", Decision::Denied(DenyReason::Mutating)),
    ("GetConsoleAuthorizationConfiguration", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members targetId, scope, consoleAuthorizationEnabled. Pinned request/ownership contract: Get console authorization configuration with automatic scope detection")),
    ("GetResourcePolicy", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members signinResourceBasedPolicy. Pinned request/ownership contract: Retrieve the account's consolidated SignIn resource-based policy")),
    // Full modeled response contains authentication/secret material . Pinned purpose: Inspects the metadata and active state of an OAuth 2.0 access token or refresh token. Implements RFC 7662 OAuth 2.0 Token Introspection over a SigV4-authenticated endpoint. Returns the claims associated with a token issued by AWS Sign-In.. Projection cannot authorize this call.
    ("IntrospectOAuth2TokenWithIAM", Decision::Denied(DenyReason::Secret)),
    ("ListResourcePermissionStatements", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members permissionStatements, nextToken. Pinned request/ownership contract: Retrieve all permission statements in the account's SignIn resource-based policy")),
    // Changes service state rather than reading inventory. Pinned purpose: Enable console authorization configuration with automatic scope detection
    ("PutConsoleAuthorizationConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Create a permission statement in the account's SignIn resource-based policy
    ("PutResourcePermissionStatement", Decision::Denied(DenyReason::Mutating)),
    // Full modeled response contains authentication/secret material . Pinned purpose: Revokes an OAuth 2.0 refresh token and its associated token chain.. Projection cannot authorize this call.
    ("RevokeOAuth2TokenWithIAM", Decision::Denied(DenyReason::Secret)),
];
